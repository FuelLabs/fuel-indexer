use chrono::Utc;
use clap::Parser;
use duct::cmd;
use fuel_indexer_lib::{config::IndexerConfig, manifest::Manifest, utils::init_logging};
use reqwest::{
    header::{HeaderMap, CONTENT_TYPE},
    Client,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    fs::canonicalize,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::time::{sleep, Duration};

#[derive(Debug, Parser, Clone)]
#[clap(name = "fuel-indexer-qa", about = "Fuel indexer QA suite.")]
struct Args {
    #[clap(short, long, default_value = "10", help = "Number of runs to perform.")]
    pub runs: String,
    #[clap(
        short,
        long,
        default_value = "25000",
        help = "Number of blocks to index during run."
    )]
    pub blocks: String,
    #[clap(short, long, help = "Network at which to connect.")]
    pub network: String,
}

#[derive(Debug)]
struct StatManager {
    pub runs: Vec<RunStat>,
    pub system: String,
}

impl StatManager {
    pub fn new() -> Self {
        let system = Command::new("uname")
            .arg("-somr")
            .output()
            .unwrap()
            .stdout
            .iter()
            .map(|b| *b as char)
            .filter_map(|c| if c == '\n' { None } else { Some(c) })
            .collect::<String>();
        Self {
            runs: Vec::new(),
            system,
        }
    }

    pub fn add_run(&mut self, run: RunStat) {
        self.runs.push(run);
    }

    pub fn report(self) {
        let Self { runs, system } = self;

        let reports = runs
            .iter()
            .map(|run| run.report())
            .collect::<Vec<String>>()
            .join("\n");

        let date = Utc::now().format("%Y-%m-%d").to_string();
        let host = Command::new("hostname")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap()
            .stdout
            .iter()
            .map(|b| *b as char)
            .collect::<String>();

        let host = host.trim_end_matches('\n').to_string();

        let runtime = runs.iter().map(|run| run.runtime).sum::<u64>() / 60;
        let missing_blocks = runs.iter().map(|run| run.missing_blocks).sum::<u64>();
        let avg_memory = runs.iter().map(|run| run.avg_mem()).sum::<u64>() as f64
            / runs.len() as f64
            / 1000.;
        let avg_cpu =
            runs.iter().map(|run| run.avg_cpu()).sum::<f64>() / runs.len() as f64;
        let avg_blocks_per_sec =
            runs.iter().map(|run| run.blocks_per_sec).sum::<f64>() / runs.len() as f64;
        let index_size = runs.iter().map(|run| run.index_size).sum::<u64>() as f64
            / runs.len() as f64
            / 1000.;

        let branch = Command::new("git")
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap()
            .stdout
            .iter()
            .map(|b| *b as char)
            .collect::<String>();

        let branch = branch.trim_end_matches('\n').to_string();

        let report = format!(
            r#"
system: {system}
date: {date}
host: {host}
branch: {branch}
runtime: {runtime:.1} minutes
missing blocks: {missing_blocks}
avg memory: {avg_memory:.1}kB
avg cpu: {avg_cpu:.1}%
avg blocks/sec: {avg_blocks_per_sec:.1}
index size: {index_size:.1}kB per block

----------------

{reports}
"#
        );

        println!("{}", report);

        let path = PathBuf::from(format!("{}-{}.indexer-qa.txt", host, date));
        let f = std::fs::File::create(&path).unwrap();
        let mut f = std::io::BufWriter::new(f);
        f.write_all(report.as_bytes()).unwrap();
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct RunStat {
    pub id: usize,
    pub start_block: u32,
    pub end_block: u32,
    pub mem: Vec<u64>,
    pub cpu: Vec<f64>,
    pub blocks_per_sec: f64,
    pub index_size: u64,
    pub missing_blocks: u64,
    pub runtime: u64,
}

impl RunStat {
    pub fn new(id: usize, start_block: u32, end_block: u32) -> Self {
        Self {
            id,
            start_block,
            end_block,
            mem: Vec::new(),
            cpu: Vec::new(),
            index_size: 0,
            blocks_per_sec: 0.0,
            missing_blocks: 0,
            runtime: 0,
        }
    }

    pub fn tick(&mut self) {
        // `sort` will occassionally panic, so just use the last iteration's value
        let bytes = 1500;
        let mem = match record_mem_usage() {
            Ok(v) => {
                if v.is_empty() {
                    self.mem.last().unwrap_or(&bytes).to_owned()
                } else {
                    v.parse::<u64>().unwrap_or(bytes)
                }
            }
            Err(_) => self.mem.last().unwrap_or(&bytes).clone(),
        };
        self.mem.push(mem);
        self.cpu.push(record_cpu_usage());
    }

    fn avg_mem(&self) -> u64 {
        self.mem.iter().sum::<u64>() / self.mem.len() as u64
    }

    fn stdv_mem(&self) -> f64 {
        let avg = self.avg_mem() as f64;
        let sum = self
            .mem
            .iter()
            .map(|x| (*x as f64 - avg).powi(2))
            .sum::<f64>();
        (sum / self.mem.len() as f64).sqrt()
    }

    fn stdv_cpu(&self) -> f64 {
        let avg = self.avg_cpu();
        let sum = self.cpu.iter().map(|x| (*x - avg).powi(2)).sum::<f64>();
        (sum / self.cpu.len() as f64).sqrt()
    }

    fn avg_cpu(&self) -> f64 {
        self.cpu.iter().sum::<f64>() / self.cpu.len() as f64
    }

    pub fn measure_performance(&mut self, runtime: u64) {
        let end_block = self.end_block;
        let start_block = self.start_block;
        let expected_count = end_block - start_block;
        let output = Command::new("psql")
            .arg("-U")
            .arg("postgres")
            .arg("-c")
            .arg(&format!(
                "SELECT COUNT(*) FROM fuellabs_explorer.header WHERE height >= {} AND height < {}", self.start_block, self.end_block
            ))
            .arg("--no-align")
            .arg("--tuples-only")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let output = output.wait_with_output().unwrap();
        let output = String::from_utf8(output.stdout).unwrap();
        let output = output.trim();
        let mut output = output.parse::<u32>().unwrap_or(0);

        // Start block and end block are inclusive.
        //
        // Because of the way the cursor's starting point is implemented in
        // `fuel_indexer::executor::run_executor`, when we start from Block #0,
        // our output will actually be 1 block less than the expected output.
        //
        // This only happens when we start from Block #0 (thus run #1)
        if self.id == 1 {
            output += 1;
        }

        if output != expected_count {
            tracing::error!("Run {} does not have expected number of indexed blocks. Expected: {}, Actual: {}", self.id, expected_count, output);
        }

        let index_size = Command::new("psql")
            .arg("-U")
            .arg("postgres")
            .arg("-c")
            .arg(&format!(
r#"
SELECT cast(sum(table_size) / cast((select count(*) from fuellabs_explorer.block) as float) as integer)
FROM (
  SELECT pg_catalog.pg_namespace.nspname as schema_name,
         pg_relation_size(pg_catalog.pg_class.oid) as table_size,
         sum(pg_relation_size(pg_catalog.pg_class.oid)) over () as database_size
  FROM   pg_catalog.pg_class
     JOIN pg_catalog.pg_namespace ON relnamespace = pg_catalog.pg_namespace.oid
) t
WHERE schema_name = 'fuellabs_explorer';
"#
            ))
            .arg("--no-align")
            .arg("--tuples-only")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let index_size = index_size.wait_with_output().unwrap();
        let index_size = String::from_utf8(index_size.stdout).unwrap();
        let index_size = index_size.trim();
        let index_size = index_size.parse::<f64>().unwrap_or(0.);
        self.index_size = index_size as u64;

        self.missing_blocks = expected_count as u64 - output as u64;
        self.blocks_per_sec = output as f64 / runtime as f64;
        self.runtime = runtime;
    }

    pub fn report(&self) -> String {
        let Self {
            id,
            start_block,
            end_block,
            blocks_per_sec,
            missing_blocks,
            runtime,
            ..
        } = self;

        let avg_mem = self.avg_mem() as f64 / 1000.;
        let stdv_cpu = self.stdv_cpu();
        let avg_cpu = self.avg_cpu();
        let stdv_mem = self.stdv_mem() / 1000.;
        let runtime = *runtime as f64 / 60.;
        let block_size = self.index_size as f64 / 1000.;

        format!(
            r#"
run: {id}
    runtime:        {runtime:.1} minutes
    start block:    {start_block}
    end block:      {end_block}
    avg memory:     {avg_mem:.1}kB
    stdv memory:    {stdv_mem:.1}kB
    avg cpu:        {avg_cpu:.1}%
    stdv cpu:       {stdv_cpu:.1}%
    missing blocks: {missing_blocks}
    blocks/sec:     {blocks_per_sec:.1}
    index size:     {block_size}kB per block"#
        )
    }
}

#[derive(Serialize, Deserialize)]
struct BaseChain {
    #[serde(rename = "baseChainHeight")]
    pub base_chain_height: String,
}

#[derive(Serialize, Deserialize)]
struct Chain {
    pub chain: BaseChain,
}

#[derive(Serialize, Deserialize)]
struct BlockHeightResponse {
    pub data: Chain,
}

#[derive(Serialize, Deserialize)]
struct BlockHeightRequest {
    pub query: String,
}

#[derive(Debug)]
enum Network {
    Beta4,
    Mainnet,
}

impl From<String> for Network {
    fn from(s: String) -> Self {
        match s.as_str() {
            "beta-4.fuel.network" => Network::Beta4,
            "mainnet" => Network::Mainnet,
            _ => panic!("Invalid network"),
        }
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Beta4 => write!(f, "beta-4.fuel.network"),
            Network::Mainnet => write!(f, "mainnet"),
        }
    }
}

fn part_range(end: u32, num_parts: u32) -> Vec<u32> {
    let start = 0;
    let len = end - start;
    let part_size = len / num_parts;
    let mut parts = Vec::new();
    let mut curr = start;
    for _ in 0..num_parts {
        let curr_end = curr + part_size;
        parts.push(curr);
        curr = curr_end;
    }
    parts
}

async fn get_start_blocks(
    network: Network,
    num_runs: u32,
    blocks_per_run: u32,
) -> Vec<u32> {
    let head = get_chain_head(&network).await - blocks_per_run;
    tracing::info!("Chain head is currently at block number {head}");
    part_range(head, num_runs)
}

async fn get_chain_head(network: &Network) -> u32 {
    let uri = format!("https://{}/graphql", network);

    let headers =
        HeaderMap::from_iter([(CONTENT_TYPE, "application/json".parse().unwrap())]);
    let body = BlockHeightRequest {
        query: "query {\n  chain {\n    baseChainHeight\n  }\n}".to_string(),
    };

    let res = Client::new()
        .post(&uri)
        .headers(headers)
        .json(&body)
        .send()
        .await
        .unwrap();

    let resp: BlockHeightResponse = res.json().await.unwrap();
    resp.data.chain.base_chain_height.parse().unwrap()
}

fn check_if_run_is_finished(
    run: usize,
    num_runs: u32,
    start: u32,
    end: u32,
    blocks_per_run: u32,
) -> bool {
    let proc = Command::new("psql")
        .arg("-U")
        .arg("postgres")
        .arg("-c")
        .arg(&format!(
            "SELECT COUNT(*) FROM fuellabs_explorer.header WHERE height >= {start} AND height < {end}"
        ))
        .arg("--no-align")
        .arg("--tuples-only")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let output = proc.wait_with_output().unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    let output = output.trim();
    let output = output.parse::<u32>().unwrap_or(0);

    // Start block and end block are inclusive.
    //
    // Because of the way the cursor's starting point is implemented in
    // `fuel_indexer::executor::run_executor`, when we start from Block #0,
    // our output will actually be 1 block less than the expected output.
    //
    // This only happens when we start from Block #0 (thus run #1)
    if run == 1 {
        return output + 1 == blocks_per_run;
    }

    let diff = end - start;
    let progress = output as f64 / diff as f64 * 100.0;

    print!(
        "\r{}",
        format!(
            r#"Run({run}/{num_runs}) | Start: Block#{start} | Progress: {output}/{diff} ({progress:.1}%)"#
        )
    );
    std::io::stdout().flush().unwrap();

    output == blocks_per_run
}

fn record_cpu_usage() -> f64 {
    let output = cmd!("ps", "aux")
        .pipe(cmd!("grep", "fuel-indexer"))
        .pipe(cmd!("sort", "-nr"))
        .pipe(cmd!("tail", "-n", "1"))
        .pipe(cmd!("awk", "{{print $4}}"))
        .stdout_capture()
        .read()
        .unwrap_or_default();

    output.trim().parse::<f64>().unwrap()
}

fn record_mem_usage() -> Result<String, std::io::Error> {
    cmd!("ps", "x", "-o", "rss,vsz,command")
        .pipe(cmd!("grep", "fuel-indexer"))
        .pipe(cmd!("sort", "-nr"))
        .pipe(cmd!("head", "-n", "1"))
        .pipe(cmd!("awk", "{print $1}"))
        .stdout_capture()
        .read()
}

#[tokio::main]
async fn main() {
    let opts = Args::from_args();
    let config = IndexerConfig::default();

    let num_runs = opts.runs.parse::<u32>().unwrap();
    let blocks_per_run = opts.blocks.parse::<u32>().unwrap();

    init_logging(&config).await.unwrap();

    let root = std::env::current_dir().unwrap();
    let explorer_root = canonicalize(
        root.join("examples")
            .join("fuel-explorer")
            .join("fuel-explorer"),
    )
    .unwrap();

    let mani_path = explorer_root.join("fuel_explorer.manifest.yaml");

    let _proc = Command::new("forc-index")
        .arg("start")
        .arg("--run-migrations")
        .arg("--fuel-node-host")
        .arg("beta-4.fuel.network")
        .arg("--fuel-node-port")
        .arg("80")
        .arg("--replace-indexer")
        .arg("--allow-non-sequential-blocks")
        .spawn()
        .unwrap();

    let start_blocks =
        get_start_blocks(opts.network.into(), num_runs, blocks_per_run).await;
    tracing::info!(
        "Performing {num_runs} runs, indexing {blocks_per_run} blocks per run."
    );
    tracing::info!("Start blocks: {start_blocks:?}");
    let manifest = Manifest::from_file(&mani_path).unwrap();
    let mut stats = StatManager::new();

    for (i, start_block) in start_blocks.iter().enumerate() {
        let start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let run = i + 1;
        let end_block = start_block + blocks_per_run;
        let mut run_stats = RunStat::new(run, *start_block, end_block);

        let mut manifest = manifest.clone();

        tracing::info!("Run {run} will index block #{start_block} - #{end_block}",);
        manifest.set_start_block(*start_block);

        manifest.set_end_block(end_block);
        let _ = manifest.write(&mani_path).unwrap();

        sleep(Duration::from_secs(1)).await;

        let mut proc = Command::new("forc-index")
            .arg("deploy")
            .arg("--path")
            .arg(&explorer_root.to_str().unwrap())
            .arg("--replace-indexer")
            .spawn()
            .unwrap();

        let _ = proc.wait().unwrap();

        sleep(Duration::from_secs(1)).await;

        let mut finished = false;
        while !finished {
            finished = check_if_run_is_finished(
                run,
                num_runs,
                *start_block,
                end_block,
                blocks_per_run,
            );
            run_stats.tick();
            sleep(Duration::from_secs(1)).await;
        }

        // TODO: Add querying as a part of the QA suite as well

        let end = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        run_stats.measure_performance(end - start);
        println!("{}", run_stats.report());
        stats.add_run(run_stats);
    }

    stats.report();

    let _ = Command::new("forc-index").arg("kill").spawn().unwrap();
}

extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(
    abi = "fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test-abi.json",
    namespace = "fuel_indexer_test",
    identifier = "index1",
    schema = "./../../assets/fuel_indexer_test.graphql"
)]
mod fuel_indexer_test {
    fn fuel_indexer_test_ping(ping: Ping) {
        Logger::info("fuel_indexer_test_ping handling a Ping event.");

        let message: String = ping.message.into();

        let mut bytes: [u8; 32] = [0u8; 32];
        bytes.copy_from_slice(&message.as_bytes()[..32]);

        let entity = Message {
            id: ping.id,
            ping: ping.value,
            pong: 456,
            message: Bytes32::from(bytes),
        };

        entity.save();
    }

    fn fuel_indexer_test_transfer(transfer: Transfer) {
        Logger::info("fuel_indexer_test_transfer handling Transfer event.");

        let entity = TransferEntity {
            id: 1,
            contract_id: transfer.contract_id,
            to: transfer.to,
            amount: 1,
            asset_id: transfer.asset_id,
        };

        entity.save();
    }

    fn fuel_indexer_test_log(log: Log) {
        Logger::info("fuel_indexer_test_log handling Log event.");

        let entity = LogEntity {
            id: 1,
            contract_id: log.contract_id,
            ra: log.ra,
            rb: log.rb,
        };

        entity.save();
    }

    fn fuel_indexer_test_logdata(logdata: Pung) {
        Logger::info("fuel_indexer_test_logdata handling Log event.");

        println!("This entity came from the LogData: {:?}", logdata);
    }
}

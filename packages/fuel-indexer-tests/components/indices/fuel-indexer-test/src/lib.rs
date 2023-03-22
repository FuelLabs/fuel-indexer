extern crate alloc;
use fuel_indexer_macros::indexer;
use fuel_indexer_plugin::prelude::*;

#[indexer(
    manifest = "packages/fuel-indexer-tests/components/indices/fuel-indexer-test/fuel_indexer_test.yaml"
)]
mod fuel_indexer_test {

    fn fuel_indexer_test_blocks(block_data: BlockData) {
        let block = Block {
            id: first8_bytes_to_u64(block_data.id),
            height: block_data.height,
            timestamp: block_data.time,
        };

        block.save();

        let input_data = r#"{"foo":"bar"}"#.to_string();

        for tx in block_data.transactions.iter() {
            let tx = Tx {
                id: first8_bytes_to_u64(tx.id),
                block: block.id,
                timestamp: block_data.time,
                input_data: Json(input_data.clone()),
            };
            tx.save();
        }
    }

    fn fuel_indexer_test_ping(ping: Ping) {
        Logger::info("fuel_indexer_test_ping handling a Ping event.");

        let entity = PingEntity {
            id: ping.id,
            value: ping.value,
            message: ping.message.to_string(),
        };

        entity.save();
    }

    fn fuel_indexer_test_u16(_ping: Ping) {
        Logger::info("fuel_indexer_test_ping handling a U16 event.");
        let u16entity = U16Entity {
            id: 9999,
            value1: 340282366920938463463374607431768211454, // 2**128-2
            value2: 170141183460469231731687303715884105727, // 2**127-1
        };

        u16entity.save();
    }

    fn fuel_indexer_test_transfer(transfer: abi::Transfer) {
        Logger::info("fuel_indexer_test_transfer handling Transfer event.");

        let abi::Transfer {
            contract_id,
            to,
            asset_id,
            amount,
            ..
        } = transfer;

        let entity = Transfer {
            id: first8_bytes_to_u64(contract_id),
            contract_id,
            recipient: to,
            amount,
            asset_id,
        };

        entity.save();
    }

    fn fuel_indexer_test_transferout(transferout: abi::TransferOut) {
        Logger::info("fuel_indexer_test_transferout handling TransferOut event.");

        let abi::TransferOut {
            contract_id,
            to,
            asset_id,
            amount,
            ..
        } = transferout;

        let entity = TransferOut {
            id: first8_bytes_to_u64(contract_id),
            contract_id,
            recipient: to,
            amount,
            asset_id,
        };

        entity.save();
    }

    fn fuel_indexer_test_log(log: abi::Log) {
        Logger::info("fuel_indexer_test_log handling Log event.");

        let abi::Log {
            contract_id,
            rb,
            ra,
            ..
        } = log;

        let entity = Log {
            id: first8_bytes_to_u64(contract_id),
            contract_id: log.contract_id,
            ra,
            rb,
        };

        entity.save();
    }

    fn fuel_indexer_test_logdata(logdata_entity: Pung) {
        Logger::info("fuel_indexer_test_logdata handling LogData event.");

        let entity = PungEntity {
            id: logdata_entity.id,
            value: logdata_entity.value,
            is_pung: logdata_entity.is_pung,
            pung_from: logdata_entity.pung_from,
        };

        entity.save();
    }

    fn fuel_indexer_test_scriptresult(scriptresult: abi::ScriptResult) {
        Logger::info("fuel_indexer_test_scriptresult handling ScriptResult event.");

        let abi::ScriptResult { result, gas_used } = scriptresult;

        let entity = ScriptResult {
            id: result,
            result,
            gas_used,
            blob: vec![1u8, 1, 1, 1, 1],
        };

        entity.save();
    }

    fn fuel_indexer_test_messageout_data(example_message: ExampleMessageStruct) {
        Logger::info("fuel_indexer_test_messageout handling MessageOut event");

        let entity = MessageEntity {
            id: example_message.id,
            message: example_message.message.to_string(),
        };

        entity.save();
    }

    fn fuel_indexer_test_messageout(messageout: abi::MessageOut) {
        Logger::info("fuel_indexer_test_messageout handling MessageOut event");

        let abi::MessageOut {
            sender,
            message_id,
            recipient,
            amount,
            nonce,
            len,
            digest,
            ..
        } = messageout;

        let entity = MessageOut {
            id: first8_bytes_to_u64(message_id),
            sender,
            recipient,
            amount,
            nonce,
            len,
            digest,
        };

        entity.save();
    }

    fn fuel_indexer_test_callreturn(pungentity: Pung) {
        Logger::info("fuel_indexer_test_callreturn handling Pung event.");

        let entity = PungEntity {
            id: pungentity.id,
            value: pungentity.value,
            is_pung: pungentity.is_pung,
            pung_from: pungentity.pung_from,
        };

        entity.save();
    }

    fn fuel_indexer_test_multiargs(
        pung: Pung,
        pong: Pong,
        ping: Ping,
        block_data: BlockData,
    ) {
        Logger::info("fuel_indexer_test_multiargs handling Pung, Pong, Ping, and BlockData events.");

        let block = Block {
            id: first8_bytes_to_u64(block_data.id),
            height: block_data.height,
            timestamp: block_data.time,
        };

        block.save();

        let pu = PungEntity {
            id: pung.id,
            value: pung.value,
            is_pung: pung.is_pung,
            pung_from: pung.pung_from,
        };

        pu.save();

        let po = PongEntity {
            id: pong.id,
            value: pong.value,
        };

        po.save();

        let pi = PingEntity {
            id: ping.id,
            value: ping.value,
            message: ping.message.to_string(),
        };

        pi.save();
    }

    fn fuel_indexer_test_optional_schema_fields(optional: Ping) {
        Logger::info("fuel_indexer_test_optional_schema_fields handling Ping event and setting optional fields.");

        let entity = OptionEntity {
            id: optional.id,
            int_required: 100,
            int_optional_some: Some(999),
            addr_optional_none: None,
        };

        entity.save();
    }

    fn fuel_indexer_test_tuple(
        event: ComplexTupleStruct,
        logdata_entity: SimpleTupleStruct,
    ) {
        Logger::info("fuel_indexer_test_tuple handling ComplexTupleStruct and SimpleTupleStruct events.");
        let data: (u32, (u64, bool, (SizedAsciiString<5>, TupleStructItem))) = event.data;
        let entity = TupleEntity {
            id: data.1 .0,
            complex_a: data.1 .2 .0.to_string(),
            complex_b: data.1 .2 .1.id,
            simple_a: logdata_entity.data.2.to_string(),
        };
        entity.save();
    }

    fn fuel_indexer_test_pure_function(call: abi::Call) {
        Logger::info("fuel_indexer_test_tuple handling Call event.");

        let abi::Call {
            contract_id,
            to,
            asset_id,
            gas,
            fn_name,
            amount,
        } = call;

        let entity = CallEntity {
            id: 123,
            contract_id,
            callee: to,
            asset_id,
            gas,
            fn_name,
            amount,
        };

        entity.save();
    }

    fn fuel_indexer_test_deeply_nested_schema_fields(_deeply_nested: SimpleQueryStruct) {
        Logger::info("fuel_indexer_test_deeply_nested_schema_fields handling DeeplyNestedQueryTestStruct event.");

        let genre1 = Genre {
            id: 1,
            name: "horror".to_string(),
        };
        let genre2 = Genre {
            id: 2,
            name: "mystery".to_string(),
        };
        let genre3 = Genre {
            id: 3,
            name: "business".to_string(),
        };

        genre1.save();
        genre2.save();
        genre3.save();

        let person1 = Person {
            id: 1,
            name: "Ava".to_string(),
        };
        let person2 = Person {
            id: 2,
            name: "Noel".to_string(),
        };
        let person3 = Person {
            id: 3,
            name: "Julio".to_string(),
        };

        person1.save();
        person2.save();
        person3.save();

        let planet = Planet {
            id: 1,
            name: "Earth".to_string(),
        };

        planet.save();

        let continent1 = Continent {
            id: 1,
            name: "North America".to_string(),
            planet: planet.id,
        };

        let continent2 = Continent {
            id: 2,
            name: "Europe".to_string(),
            planet: planet.id,
        };

        continent1.save();
        continent2.save();

        let country1 = Country {
            id: 1,
            name: "Jamaica".to_string(),
            continent: continent1.id,
        };

        let country2 = Country {
            id: 2,
            name: "France".to_string(),
            continent: continent2.id,
        };

        let country3 = Country {
            id: 3,
            name: "Ireland".to_string(),
            continent: continent2.id,
        };

        country1.save();
        country2.save();
        country3.save();

        let region1 = Region {
            id: 1,
            name: "Westmoreland".to_string(),
            country: country1.id,
        };

        let region2 = Region {
            id: 2,
            name: "Corsica".to_string(),
            country: country2.id,
        };

        let region3 = Region {
            id: 3,
            name: "Donegal".to_string(),
            country: country3.id,
        };

        region1.save();
        region2.save();
        region3.save();

        let city1 = City {
            id: 1,
            name: "Savanna-la-Mar".to_string(),
            region: region1.id,
        };
        let city2 = City {
            id: 2,
            name: "Ajaccio".to_string(),
            region: region2.id,
        };
        let city3 = City {
            id: 3,
            name: "Letterkenny".to_string(),
            region: region3.id,
        };

        city1.save();
        city2.save();
        city3.save();

        let author1 = Author {
            id: 1,
            name: "Brian".to_string(),
            genre: genre1.id,
        };
        let author2 = Author {
            id: 2,
            name: "James".to_string(),
            genre: genre2.id,
        };
        let author3 = Author {
            id: 3,
            name: "Susan".to_string(),
            genre: genre3.id,
        };

        author1.save();
        author2.save();
        author3.save();

        let library1 = Library {
            id: 1,
            name: "Scholar Library".to_string(),
            city: city1.id,
        };
        let library2 = Library {
            id: 2,
            name: "Ronoke Library".to_string(),
            city: city2.id,
        };
        let library3 = Library {
            id: 3,
            name: "Scholastic Library".to_string(),
            city: city3.id,
        };

        library1.save();
        library2.save();
        library3.save();

        let book1 = Book {
            id: 1,
            name: "Gone with the Wind".to_string(),
            author: author1.id,
            library: library1.id,
            genre: genre1.id,
        };
        let book2 = Book {
            id: 2,
            name: "Othello".to_string(),
            author: author2.id,
            library: library2.id,
            genre: genre2.id,
        };
        let book3 = Book {
            id: 3,
            name: "Cyberpunk 2021".to_string(),
            author: author3.id,
            library: library3.id,
            genre: genre3.id,
        };

        book1.save();
        book2.save();
        book3.save();

        let sponsor1 = Sponsor {
            id: 1,
            name: "Fuel Labs".to_string(),
            amount: 100,
            representative: person1.id,
        };

        let sponsor2 = Sponsor {
            id: 2,
            name: "Fuel Labs, Part Two".to_string(),
            amount: 200,
            representative: person2.id,
        };

        let sponsor3 = Sponsor {
            id: 3,
            name: "Fuel Labs, Yet A Third".to_string(),
            amount: 300,
            representative: person3.id,
        };

        sponsor1.save();
        sponsor2.save();
        sponsor3.save();

        let bookclub1 = BookClub {
            id: 1,
            book: book1.id,
            member: person1.id,
            corporate_sponsor: sponsor1.name,
        };

        let bookclub2 = BookClub {
            id: 2,
            book: book2.id,
            member: person2.id,
            corporate_sponsor: sponsor2.name,
        };

        let bookclub3 = BookClub {
            id: 3,
            book: book3.id,
            member: person3.id,
            corporate_sponsor: sponsor3.name,
        };

        bookclub1.save();
        bookclub2.save();
        bookclub3.save();
    }

    fn fuel_indexer_test_nested_query_explicit_foreign_keys_schema_fields(
        explicit: ExplicitQueryStruct,
    ) {
        Logger::info("fuel_indexer_test_nested_query_explicit_foreign_keys_schema_fields handling ExplicitQueryTestStruct event.");

        let municipality = Municipality {
            id: explicit.id,
            name: "Republic of Indexia".to_string(),
        };

        municipality.save();

        let team = SportsTeam {
            id: explicit.id,
            name: "The Indexers".to_string(),
            municipality: municipality.name,
        };

        team.save();
    }

    fn fuel_indexer_trigger_panic(data: abi::Panic) {
        Logger::info("fuel_indexer_trigger_panic handling Panic event.");

        panic!("Panic triggered by Fuel Indexer.");
    }
}

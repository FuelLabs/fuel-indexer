extern crate alloc;

use fuel_indexer_utils::prelude::*;

#[indexer(
    manifest = "packages/fuel-indexer-tests/components/indices/fuel-indexer-test/fuel_indexer_test.yaml"
)]
mod fuel_indexer_test {

    fn foo(b: BlockData) {
        Logger::info("rrrrr");
    }

    // fn fuel_indexer_test_blocks(block_data: BlockData) {
    //     let block = BlockEntity {
    //         id: id8(block_data.id),
    //         height: block_data.height,
    //         timestamp: block_data.time,
    //     };

    //     block.save();

    //     let input_data = r#"{"foo":"bar"}"#.to_string();

    //     for tx in block_data.transactions.iter() {
    //         let tx = TxEntity {
    //             id: id8(tx.id),
    //             block: block.id,
    //             timestamp: block_data.time,
    //             input_data: Json(input_data.clone()),
    //         };
    //         tx.save();
    //     }
    // }

    // fn fuel_indexer_test_ping(ping: Ping) {
    //     info!("fuel_indexer_test_ping handling a Ping event: {:?}", ping);

    //     let entity = PingEntity {
    //         id: ping.id,
    //         value: ping.value,
    //         message: ping.message.to_string(),
    //     };

    //     entity.save();
    // }

    // fn fuel_indexer_test_u16(_ping: Ping) {
    //     info!("fuel_indexer_test_ping handling a U16 event.");
    //     let u16entity = U16Entity {
    //         id: 9999,
    //         value1: 340282366920938463463374607431768211454, // 2**128-2
    //         value2: 170141183460469231731687303715884105727, // 2**127-1
    //     };

    //     u16entity.save();
    // }

    // fn fuel_indexer_test_transfer(transfer: Transfer) {
    //     info!(
    //         "fuel_indexer_test_transfer handling Transfer event: {:?}",
    //         transfer
    //     );

    //     let Transfer {
    //         contract_id,
    //         to,
    //         asset_id,
    //         amount,
    //         ..
    //     } = transfer;

    //     let entity = TransferEntity {
    //         id: id8(contract_id),
    //         contract_id,
    //         recipient: to,
    //         amount,
    //         asset_id,
    //     };

    //     entity.save();
    // }

    // fn fuel_indexer_test_transferout(transferout: TransferOut) {
    //     info!(
    //         "fuel_indexer_test_transferout handling TransferOut event: {:?}",
    //         transferout
    //     );

    //     let TransferOut {
    //         contract_id,
    //         to,
    //         asset_id,
    //         amount,
    //         ..
    //     } = transferout;

    //     let entity = TransferOutEntity {
    //         id: id8(contract_id),
    //         contract_id,
    //         recipient: to,
    //         amount,
    //         asset_id,
    //     };

    //     entity.save();
    // }

    // fn fuel_indexer_test_log(log: Log) {
    //     info!("fuel_indexer_test_log handling Log event.");

    //     let Log {
    //         contract_id,
    //         rb,
    //         ra,
    //         ..
    //     } = log;

    //     let entity = LogEntity {
    //         id: id8(contract_id),
    //         contract_id: log.contract_id,
    //         ra,
    //         rb,
    //     };

    //     entity.save();
    // }

    // fn fuel_indexer_test_logdata(logdata_entity: Pung) {
    //     info!("fuel_indexer_test_logdata handling LogData event.");

    //     let entity = PungEntity {
    //         id: logdata_entity.id,
    //         value: logdata_entity.value,
    //         is_pung: logdata_entity.is_pung,
    //         pung_from: logdata_entity.pung_from,
    //     };

    //     entity.save();
    // }

    // fn fuel_indexer_test_scriptresult(scriptresult: ScriptResult) {
    //     info!("fuel_indexer_test_scriptresult handling ScriptResult event: result={}, gas_used={}", scriptresult.result, scriptresult.gas_used);

    //     let ScriptResult { result, gas_used } = scriptresult;

    //     let entity = ScriptResultEntity {
    //         id: result,
    //         result,
    //         gas_used,
    //         blob: vec![1u8, 1, 1, 1, 1].into(),
    //     };

    //     entity.save();
    // }

    // fn fuel_indexer_test_messageout_data(example_message: ExampleMessageStruct) {
    //     info!("fuel_indexer_test_messageout_data handling MessageOut data event");

    //     let entity = MessageEntity {
    //         id: example_message.id,
    //         message: example_message.message.to_string(),
    //     };

    //     entity.save();
    // }

    // fn fuel_indexer_test_messageout(messageout: MessageOut) {
    //     info!("fuel_indexer_test_messageout handling MessageOut event");

    //     let MessageOut {
    //         sender,
    //         message_id,
    //         recipient,
    //         amount,
    //         nonce,
    //         len,
    //         digest,
    //         ..
    //     } = messageout;

    //     let entity = MessageOutEntity {
    //         id: id8(message_id),
    //         message_id,
    //         sender,
    //         recipient,
    //         amount,
    //         nonce,
    //         len,
    //         digest,
    //     };

    //     entity.save();
    // }

    // fn fuel_indexer_test_callreturn(pungentity: Pung) {
    //     info!("fuel_indexer_test_callreturn handling Pung event.");

    //     let entity = PungEntity {
    //         id: pungentity.id,
    //         value: pungentity.value,
    //         is_pung: pungentity.is_pung,
    //         pung_from: pungentity.pung_from,
    //     };

    //     entity.save();
    // }

    // fn fuel_indexer_test_multiargs(
    //     pung: Pung,
    //     pong: Pong,
    //     ping: Ping,
    //     block_data: BlockData,
    // ) {
    //     info!(
    //         "fuel_indexer_test_multiargs handling Pung, Pong, Ping, and BlockData events."
    //     );

    //     let block = BlockEntity {
    //         id: id8(block_data.id),
    //         height: block_data.height,
    //         timestamp: block_data.time,
    //     };

    //     block.save();

    //     let pu = PungEntity {
    //         id: pung.id,
    //         value: pung.value,
    //         is_pung: pung.is_pung,
    //         pung_from: pung.pung_from,
    //     };

    //     pu.save();

    //     let po = PongEntity {
    //         id: pong.id,
    //         value: pong.value,
    //     };

    //     po.save();

    //     let pi = PingEntity {
    //         id: ping.id,
    //         value: ping.value,
    //         message: ping.message.to_string(),
    //     };

    //     pi.save();
    // }

    // fn fuel_indexer_test_optional_schema_fields(optional: Ping) {
    //     info!("fuel_indexer_test_optional_schema_fields handling Ping event and setting optional fields.");

    //     let entity = OptionEntity {
    //         id: optional.id,
    //         int_required: 100,
    //         int_optional_some: Some(999),
    //         addr_optional_none: None,
    //     };

    //     entity.save();
    // }

    // fn fuel_indexer_test_tuple(
    //     event: ComplexTupleStruct,
    //     logdata_entity: SimpleTupleStruct,
    // ) {
    //     info!("fuel_indexer_test_tuple handling ComplexTupleStruct and SimpleTupleStruct events.");
    //     let data: (u32, (u64, bool, (SizedAsciiString<5>, TupleStructItem))) = event.data;
    //     let entity = TupleEntity {
    //         id: data.1 .0,
    //         complex_a: data.1 .2 .0.to_string(),
    //         complex_b: data.1 .2 .1.id,
    //         simple_a: logdata_entity.data.2.to_string(),
    //     };
    //     entity.save();
    // }

    // fn fuel_indexer_test_pure_function(call: Call) {
    //     info!("fuel_indexer_test_tuple handling Call event.");

    //     let Call {
    //         contract_id,
    //         to,
    //         asset_id,
    //         gas,
    //         fn_name,
    //         amount,
    //     } = call;

    //     let entity = CallEntity {
    //         id: 123,
    //         contract_id,
    //         callee: to,
    //         asset_id,
    //         gas,
    //         fn_name,
    //         amount,
    //     };

    //     entity.save();
    // }

    // fn fuel_indexer_test_deeply_nested_schema_fields(_deeply_nested: SimpleQueryStruct) {
    //     info!("fuel_indexer_test_deeply_nested_schema_fields handling DeeplyNestedQueryTestStruct event.");

    //     let genre1 = Genre {
    //         id: 1,
    //         name: "horror".to_string(),
    //     };
    //     let genre2 = Genre {
    //         id: 2,
    //         name: "mystery".to_string(),
    //     };
    //     let genre3 = Genre {
    //         id: 3,
    //         name: "business".to_string(),
    //     };

    //     genre1.save();
    //     genre2.save();
    //     genre3.save();

    //     let person1 = Person {
    //         id: 1,
    //         name: "Ava".to_string(),
    //     };
    //     let person2 = Person {
    //         id: 2,
    //         name: "Noel".to_string(),
    //     };
    //     let person3 = Person {
    //         id: 3,
    //         name: "Julio".to_string(),
    //     };

    //     person1.save();
    //     person2.save();
    //     person3.save();

    //     let planet = Planet {
    //         id: 1,
    //         name: "Earth".to_string(),
    //     };

    //     planet.save();

    //     let continent1 = Continent {
    //         id: 1,
    //         name: "North America".to_string(),
    //         planet: planet.id,
    //     };

    //     let continent2 = Continent {
    //         id: 2,
    //         name: "Europe".to_string(),
    //         planet: planet.id,
    //     };

    //     continent1.save();
    //     continent2.save();

    //     let country1 = Country {
    //         id: 1,
    //         name: "Jamaica".to_string(),
    //         continent: continent1.id,
    //     };

    //     let country2 = Country {
    //         id: 2,
    //         name: "France".to_string(),
    //         continent: continent2.id,
    //     };

    //     let country3 = Country {
    //         id: 3,
    //         name: "Ireland".to_string(),
    //         continent: continent2.id,
    //     };

    //     country1.save();
    //     country2.save();
    //     country3.save();

    //     let region1 = Region {
    //         id: 1,
    //         name: "Westmoreland".to_string(),
    //         country: country1.id,
    //     };

    //     let region2 = Region {
    //         id: 2,
    //         name: "Corsica".to_string(),
    //         country: country2.id,
    //     };

    //     let region3 = Region {
    //         id: 3,
    //         name: "Donegal".to_string(),
    //         country: country3.id,
    //     };

    //     region1.save();
    //     region2.save();
    //     region3.save();

    //     let city1 = City {
    //         id: 1,
    //         name: "Savanna-la-Mar".to_string(),
    //         region: region1.id,
    //     };
    //     let city2 = City {
    //         id: 2,
    //         name: "Ajaccio".to_string(),
    //         region: region2.id,
    //     };
    //     let city3 = City {
    //         id: 3,
    //         name: "Letterkenny".to_string(),
    //         region: region3.id,
    //     };

    //     city1.save();
    //     city2.save();
    //     city3.save();

    //     let author1 = Author {
    //         id: 1,
    //         name: "Brian".to_string(),
    //         genre: genre1.id,
    //     };
    //     let author2 = Author {
    //         id: 2,
    //         name: "James".to_string(),
    //         genre: genre2.id,
    //     };
    //     let author3 = Author {
    //         id: 3,
    //         name: "Susan".to_string(),
    //         genre: genre3.id,
    //     };

    //     author1.save();
    //     author2.save();
    //     author3.save();

    //     let library1 = Library {
    //         id: 1,
    //         name: "Scholar Library".to_string(),
    //         city: city1.id,
    //     };
    //     let library2 = Library {
    //         id: 2,
    //         name: "Ronoke Library".to_string(),
    //         city: city2.id,
    //     };
    //     let library3 = Library {
    //         id: 3,
    //         name: "Scholastic Library".to_string(),
    //         city: city3.id,
    //     };

    //     library1.save();
    //     library2.save();
    //     library3.save();

    //     let book1 = Book {
    //         id: 1,
    //         name: "Gone with the Wind".to_string(),
    //         author: author1.id,
    //         library: library1.id,
    //         genre: genre1.id,
    //     };
    //     let book2 = Book {
    //         id: 2,
    //         name: "Othello".to_string(),
    //         author: author2.id,
    //         library: library2.id,
    //         genre: genre2.id,
    //     };
    //     let book3 = Book {
    //         id: 3,
    //         name: "Cyberpunk 2021".to_string(),
    //         author: author3.id,
    //         library: library3.id,
    //         genre: genre3.id,
    //     };

    //     book1.save();
    //     book2.save();
    //     book3.save();

    //     let sponsor1 = Sponsor {
    //         id: 1,
    //         name: "Fuel Labs".to_string(),
    //         amount: 100,
    //         representative: person1.id,
    //     };

    //     let sponsor2 = Sponsor {
    //         id: 2,
    //         name: "Fuel Labs, Part Two".to_string(),
    //         amount: 200,
    //         representative: person2.id,
    //     };

    //     let sponsor3 = Sponsor {
    //         id: 3,
    //         name: "Fuel Labs, Yet A Third".to_string(),
    //         amount: 300,
    //         representative: person3.id,
    //     };

    //     sponsor1.save();
    //     sponsor2.save();
    //     sponsor3.save();

    //     let bookclub1 = BookClub {
    //         id: 1,
    //         book: book1.id,
    //         member: person1.id,
    //         corporate_sponsor: sponsor1.name,
    //     };

    //     let bookclub2 = BookClub {
    //         id: 2,
    //         book: book2.id,
    //         member: person2.id,
    //         corporate_sponsor: sponsor2.name,
    //     };

    //     let bookclub3 = BookClub {
    //         id: 3,
    //         book: book3.id,
    //         member: person3.id,
    //         corporate_sponsor: sponsor3.name,
    //     };

    //     bookclub1.save();
    //     bookclub2.save();
    //     bookclub3.save();
    // }

    // fn fuel_indexer_test_nested_query_explicit_foreign_keys_schema_fields(
    //     explicit: ExplicitQueryStruct,
    // ) {
    //     info!("fuel_indexer_test_nested_query_explicit_foreign_keys_schema_fields handling ExplicitQueryTestStruct event.");

    //     let municipality = Municipality {
    //         id: explicit.id,
    //         name: "Republic of Indexia".to_string(),
    //     };

    //     municipality.save();

    //     let team = SportsTeam {
    //         id: explicit.id,
    //         name: "The Indexers".to_string(),
    //         municipality: municipality.name,
    //     };

    //     team.save();
    // }

    // fn fuel_indexer_test_panic(panic: Panic) {
    //     info!("fuel_indexer_test_panic handling Panic event.");

    //     let Panic {
    //         contract_id,
    //         reason,
    //     } = panic;

    //     let panic = PanicEntity {
    //         id: 123,
    //         contract_id,
    //         reason,
    //     };

    //     panic.save();
    // }

    // fn fuel_indexer_trigger_revert(revert: Revert) {
    //     info!("fuel_indexer_trigger_revert handling trigger_revert event.");

    //     let Revert {
    //         contract_id,
    //         error_val,
    //     } = revert;

    //     let entity = RevertEntity {
    //         id: 123,
    //         contract_id,
    //         error_val,
    //     };

    //     entity.save();
    // }

    // fn fuel_indexer_test_filterable_fields(_ping: Ping) {
    //     let inner_entity1 = InnerFilterEntity {
    //         id: 1,
    //         inner_foo: "spam".to_string(),
    //         inner_bar: 100,
    //         inner_baz: 200,
    //     };

    //     let inner_entity2 = InnerFilterEntity {
    //         id: 2,
    //         inner_foo: "ham".to_string(),
    //         inner_bar: 300,
    //         inner_baz: 400,
    //     };

    //     let inner_entity3 = InnerFilterEntity {
    //         id: 3,
    //         inner_foo: "eggs".to_string(),
    //         inner_bar: 500,
    //         inner_baz: 600,
    //     };

    //     inner_entity1.save();
    //     inner_entity2.save();
    //     inner_entity3.save();

    //     let f1 = FilterEntity {
    //         id: 1,
    //         foola: "beep".to_string(),
    //         maybe_null_bar: Some(123),
    //         bazoo: 1,
    //         inner_entity: inner_entity1.id,
    //     };

    //     let f2 = FilterEntity {
    //         id: 2,
    //         foola: "boop".to_string(),
    //         maybe_null_bar: None,
    //         bazoo: 5,
    //         inner_entity: inner_entity2.id,
    //     };

    //     let f3 = FilterEntity {
    //         id: 3,
    //         foola: "blorp".to_string(),
    //         maybe_null_bar: Some(456),
    //         bazoo: 1000,
    //         inner_entity: inner_entity3.id,
    //     };

    //     f1.save();
    //     f2.save();
    //     f3.save();
    // }

    // fn fuel_indexer_trigger_enum_error(enum_error: Revert) {
    //     info!("fuel_indexer_trigger_enum_error handling trigger_enum_error event.");

    //     let Revert {
    //         contract_id,
    //         error_val,
    //     } = enum_error;

    //     let entity = EnumError {
    //         id: 42,
    //         contract_id,
    //         error_val,
    //     };

    //     entity.save();
    // }

    // fn fuel_indexer_block_explorer_types(_b: BlockData) {
    //     info!("fuel_indexer_block_explorer_types handling explorer_types event.");
    //     let e = ExplorerEntity {
    //         id: 8675309,
    //         nonce: Nonce::default(),
    //         // TOOD: Finish
    //         time: None,
    //         hex: Some(HexString::from("hello world!")),
    //         sig: Signature::default(),
    //         bytes: Bytes64::default(),
    //     };

    //     e.save();
    // }

    // #[allow(unused)]
    // fn fuel_indexer_trigger_enum(
    //     first: AnotherSimpleEnum,
    //     second: NestedEnum,
    //     third: AnotherSimpleEnum,
    // ) {
    //     info!("fuel_indexer_trigger_enum handling trigger_enum event..");

    //     let e = ComplexEnumEntity {
    //         id: 1,
    //         one: Some(EnumEntity::One.into()),
    //     };
    //     e.save();
    // }

    // fn fuel_indexer_trigger_non_indexable_type(_b: BlockData) {
    //     info!("fuel_indexer_trigger_non_indexable_type handling trigger_non_indexable_type event.");
    //     let e = UsesVirtualEntity {
    //         id: 1,
    //         name: "hello world".to_string(),
    //         no_table: VirtualEntity {
    //             name: Some("virtual".to_string()),
    //             size: 1,
    //         }
    //         .into(),
    //     };

    //     e.save();
    // }

    // fn fuel_indexer_trigger_union_type(_b: BlockData) {
    //     info!("fuel_indexer_trigger_union_type handling trigger_union_type event.");

    //     let v = VirtualUnionEntity {
    //         a: Some(2),
    //         b: None,
    //         c: Some(6),
    //         union_type: Some(UnionType::B.into()),
    //     };

    //     let vc = VirtualUnionContainerEntity {
    //         id: 1,
    //         union_entity: Some(v.into()),
    //         union_type: UnionType::B.into(),
    //     };

    //     vc.save();

    //     let e = IndexableUnionEntity {
    //         id: 1,
    //         a: Some(5),
    //         b: Some(10),
    //         c: None,
    //         union_type: Some(UnionType::A.into()),
    //     };

    //     e.save();
    // }

    fn fuel_indexer_test_trigger_list(b: BlockData) {
        info!("fuel_indexer_test_trigger_list handling trigger_list event.");

        let list_fk1 = ListFKType { id: 1, value: 1 };
        list_fk1.save();

        let list_fk2 = ListFKType { id: 2, value: 2 };
        list_fk2.save();

        let list_fk3 = ListFKType { id: 3, value: 3 };
        list_fk3.save();

        let e = ListTypeEntity {
            id: 1,
            foo: "hello world".to_string(),
            required_all: vec![list_fk1.id, list_fk2.id, list_fk3.id],
            optional_inner: vec![
                Some("hello".to_string()),
                None,
                Some("world".to_string()),
                None,
            ],
            optional_outer: Some(vec![1, 2, 3, 4, 5]),
            optional_all: Some(vec![Some(1), None, Some(3), None]),
        };

        e.save();
    }
}

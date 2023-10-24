extern crate alloc;

use fuel_indexer_utils::prelude::*;

#[indexer(
    manifest = "packages/fuel-indexer-tests/indexers/fuel-indexer-test/fuel_indexer_test.yaml"
)]
mod fuel_indexer_test {

    fn fuel_indexer_test_blocks(block_data: BlockData) {
        let block = BlockEntity::new(block_data.height, block_data.time).get_or_create();

        let input_data = r#"{"foo":"bar"}"#.to_string();

        for _tx in block_data.transactions.iter() {
            TxEntity::new(block.id.clone(), Json(input_data.clone()), block_data.time)
                .get_or_create();
        }
    }

    fn fuel_indexer_test_ping(ping: Ping) {
        info!("fuel_indexer_test_ping handling a Ping event: {:?}.", ping);

        PingEntity::new(ping.value, ping.message.to_string()).get_or_create();
    }

    fn fuel_indexer_test_u16(_ping: Ping) {
        info!("fuel_indexer_test_ping handling a U16 event.");

        U16Entity::new(
            340282366920938463463374607431768211454,
            170141183460469231731687303715884105727,
        )
        .get_or_create();
    }

    fn fuel_indexer_test_transfer(transfer: Transfer) {
        info!(
            "fuel_indexer_test_transfer handling Transfer event: {:?}",
            transfer
        );

        let Transfer {
            contract_id,
            to,
            asset_id,
            amount,
            ..
        } = transfer;

        TransferEntity::new(contract_id, to, amount, asset_id).get_or_create();
    }

    fn fuel_indexer_test_transferout(transferout: TransferOut) {
        info!(
            "fuel_indexer_test_transferout handling TransferOut event: {:?}",
            transferout
        );

        let TransferOut {
            contract_id,
            to,
            asset_id,
            amount,
            ..
        } = transferout;

        TransferOutEntity::new(contract_id, to, amount, asset_id).get_or_create();
    }

    fn fuel_indexer_test_log(log: Log) {
        info!("fuel_indexer_test_log handling Log event.");

        let Log {
            contract_id,
            rb,
            ra,
            ..
        } = log;

        LogEntity::new(contract_id, ra, rb).get_or_create();
    }

    fn fuel_indexer_test_logdata(logdata_entity: Pung) {
        info!("fuel_indexer_test_logdata handling LogData event.");

        PungEntity::new(
            logdata_entity.value,
            logdata_entity.is_pung,
            logdata_entity.pung_from,
        )
        .get_or_create();
    }

    fn fuel_indexer_test_scriptresult(scriptresult: ScriptResult) {
        info!("fuel_indexer_test_scriptresult handling ScriptResult event: result={}, gas_used={}", scriptresult.result, scriptresult.gas_used);

        let ScriptResult { result, gas_used } = scriptresult;

        ScriptResultEntity::new(result, gas_used, vec![1u8; 5].into()).get_or_create();
    }

    fn fuel_indexer_test_messageout_data(example_message: ExampleMessageStruct) {
        info!("fuel_indexer_test_messageout_data handling MessageOut data event");

        MessageEntity::new(example_message.message.to_string()).get_or_create();
    }

    fn fuel_indexer_test_messageout(messageout: MessageOut) {
        info!("fuel_indexer_test_messageout handling MessageOut event");

        let MessageOut {
            sender,
            message_id,
            recipient,
            amount,
            nonce,
            len,
            digest,
            ..
        } = messageout;

        MessageOutEntity::new(
            bytes32(message_id),
            sender,
            recipient,
            amount,
            bytes32(nonce),
            len,
            digest,
        )
        .get_or_create();
    }

    fn fuel_indexer_test_callreturn(pungentity: Pung) {
        info!("fuel_indexer_test_callreturn handling Pung event.");

        PungEntity::new(pungentity.value, pungentity.is_pung, pungentity.pung_from)
            .get_or_create();
    }

    fn fuel_indexer_test_multiargs(
        pung: Pung,
        pong: Pong,
        ping: Ping,
        block_data: BlockData,
    ) {
        info!(
            "fuel_indexer_test_multiargs handling Pung, Pong, Ping, and BlockData events."
        );

        BlockEntity::new(block_data.height, block_data.time).get_or_create();

        PungEntity::new(pung.value, pung.is_pung, pung.pung_from).get_or_create();

        PongEntity::new(pong.value).get_or_create();

        PingEntity::new(ping.value, ping.message.to_string()).get_or_create();
    }

    fn fuel_indexer_test_optional_schema_fields(_block: BlockData) {
        info!("fuel_indexer_test_optional_schema_fields handling Ping event and setting optional fields.");

        let entity = OptionEntity {
            id: uid(8675309_u32.to_le_bytes()),
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
        info!("fuel_indexer_test_tuple handling ComplexTupleStruct and SimpleTupleStruct events.");
        let data: (u32, (u64, bool, (SizedAsciiString<5>, TupleStructItem))) = event.data;

        TupleEntity::new(
            data.1 .2 .0.to_string(),
            data.1 .2 .1.id,
            logdata_entity.data.2.to_string(),
        )
        .get_or_create();
    }

    fn fuel_indexer_test_pure_function(call: Call) {
        info!("fuel_indexer_test_tuple handling Call event.");

        let Call {
            contract_id,
            to,
            asset_id,
            gas,
            fn_name,
            amount,
        } = call;

        CallEntity::new(contract_id, to, asset_id, gas, fn_name, amount).get_or_create();
    }

    fn fuel_indexer_test_deeply_nested_schema_fields(_deeply_nested: SimpleQueryStruct) {
        info!("fuel_indexer_test_deeply_nested_schema_fields handling DeeplyNestedQueryTestStruct event.");

        let genre1 = Genre::new("horror".to_string()).get_or_create();
        let genre2 = Genre::new("mystery".to_string()).get_or_create();
        let genre3 = Genre::new("business".to_string()).get_or_create();

        let author1 = Author::new("Brian".to_string(), genre1.id.clone()).get_or_create();
        let author2 = Author::new("James".to_string(), genre2.id.clone()).get_or_create();
        let author3 = Author::new("Susan".to_string(), genre3.id.clone()).get_or_create();

        let person1 = Person::new("Ava".to_string()).get_or_create();
        let person2 = Person::new("Noel".to_string()).get_or_create();
        let person3 = Person::new("Julio".to_string()).get_or_create();

        let planet = Planet::new("Earth".to_string()).get_or_create();

        let continent1 = Continent::new("North America".to_string(), planet.id.clone())
            .get_or_create();
        let continent2 = Continent::new("Europe".to_string(), planet.id).get_or_create();

        let country1 = Country::new("Jamaica".to_string(), continent1.id).get_or_create();
        let country2 =
            Country::new("France".to_string(), continent2.id.clone()).get_or_create();
        let country3 = Country::new("Ireland".to_string(), continent2.id).get_or_create();

        let region1 =
            Region::new("Westmoreland".to_string(), country1.id).get_or_create();
        let region2 = Region::new("Corsica".to_string(), country2.id).get_or_create();
        let region3 = Region::new("Donegal".to_string(), country3.id).get_or_create();

        let city1 = City::new("Savanna-la-Mar".to_string(), region1.id).get_or_create();
        let city2 = City::new("Ajaccio".to_string(), region2.id).get_or_create();
        let city3 = City::new("Letterkenny".to_string(), region3.id).get_or_create();

        let library1 =
            Library::new("Scholar Library".to_string(), city1.id).get_or_create();
        let library2 =
            Library::new("Ronoke Library".to_string(), city2.id).get_or_create();
        let library3 =
            Library::new("Scholastic Library".to_string(), city3.id).get_or_create();

        let book1 = Book::new(
            "Gone with the Wind".to_string(),
            author1.id,
            library1.id,
            genre1.id,
        )
        .get_or_create();

        let book2 = Book::new("Othello".to_string(), author2.id, library2.id, genre2.id)
            .get_or_create();

        let book3 = Book::new(
            "Cyberpunk 2021".to_string(),
            author3.id,
            library3.id,
            genre3.id,
        )
        .get_or_create();

        let sponsor1 = Sponsor::new("Fuel Labs".to_string(), 100, person1.id.clone())
            .get_or_create();

        let sponsor2 =
            Sponsor::new("Fuel Labs, Part Two".to_string(), 200, person2.id.clone())
                .get_or_create();

        let sponsor3 = Sponsor::new(
            "Fuel Labs, Yet A Third".to_string(),
            300,
            person3.id.clone(),
        )
        .get_or_create();

        let bookclub1 = BookClub {
            id: uid([1]),
            book: book1.id,
            member: person1.id,
            corporate_sponsor: sponsor1.name,
        };

        let bookclub2 = BookClub {
            id: uid([2]),
            book: book2.id,
            member: person2.id,
            corporate_sponsor: sponsor2.name,
        };

        let bookclub3 = BookClub {
            id: uid([3]),
            book: book3.id,
            member: person3.id,
            corporate_sponsor: sponsor3.name,
        };

        bookclub1.save();
        bookclub2.save();
        bookclub3.save();
    }

    fn fuel_indexer_test_nested_query_explicit_foreign_keys_schema_fields(
        _block: BlockData,
    ) {
        info!("fuel_indexer_test_nested_query_explicit_foreign_keys_schema_fields handling ExplicitQueryTestStruct event.");

        let municipality =
            Municipality::new("Republic of Indexia".to_string()).get_or_create();

        SportsTeam::new("The Indexers".to_string(), municipality.name).get_or_create();
    }

    fn fuel_indexer_test_panic(panic: Panic) {
        info!("fuel_indexer_test_panic handling Panic event.");

        let Panic {
            contract_id,
            reason,
        } = panic;

        PanicEntity::new(contract_id, reason).get_or_create();
    }

    fn fuel_indexer_trigger_revert(revert: Revert) {
        info!("fuel_indexer_trigger_revert handling trigger_revert event.");

        let Revert {
            contract_id,
            error_val,
        } = revert;

        RevertEntity::new(contract_id, error_val).get_or_create();
    }

    fn fuel_indexer_trigger_mint(mint: Mint) {
        info!("fuel_indexer_trigger_mint handling trigger_mint event.");

        let Mint {
            sub_id,
            contract_id,
            val,
            ..
        } = mint;

        MintEntity::new(sub_id, contract_id, val).get_or_create();
    }

    fn fuel_indexer_trigger_burn(burn: Burn) {
        info!("fuel_indexer_trigger_burn handling trigger_burn event.");

        let Burn {
            sub_id,
            contract_id,
            val,
            ..
        } = burn;

        BurnEntity::new(sub_id, contract_id, val).get_or_create();
    }

    fn fuel_indexer_test_filterable_fields(_ping: Ping) {
        let inner_entity1 = InnerFilterEntity {
            id: uid([1]),
            inner_foo: "spam".to_string(),
            inner_bar: 100,
            inner_baz: 200,
        };

        let inner_entity2 = InnerFilterEntity {
            id: uid([2]),
            inner_foo: "ham".to_string(),
            inner_bar: 300,
            inner_baz: 400,
        };

        let inner_entity3 = InnerFilterEntity {
            id: uid([3]),
            inner_foo: "eggs".to_string(),
            inner_bar: 500,
            inner_baz: 600,
        };

        inner_entity1.save();
        inner_entity2.save();
        inner_entity3.save();

        let f1 = FilterEntity {
            id: uid([1]),
            foola: "beep".to_string(),
            maybe_null_bar: Some(123),
            bazoo: 1,
            inner_entity: inner_entity1.id,
        };

        let f2 = FilterEntity {
            id: uid([2]),
            foola: "boop".to_string(),
            maybe_null_bar: None,
            bazoo: 5,
            inner_entity: inner_entity2.id,
        };

        let f3 = FilterEntity {
            id: uid([3]),
            foola: "blorp".to_string(),
            maybe_null_bar: Some(456),
            bazoo: 1000,
            inner_entity: inner_entity3.id,
        };

        f1.save();
        f2.save();
        f3.save();
    }

    fn fuel_indexer_trigger_enum_error(enum_error: Revert) {
        info!("fuel_indexer_trigger_enum_error handling trigger_enum_error event.");

        let Revert {
            contract_id,
            error_val,
        } = enum_error;

        EnumError::new(contract_id, error_val).get_or_create();
    }

    fn fuel_indexer_block_explorer_types(_b: BlockData) {
        info!("fuel_indexer_block_explorer_types handling explorer_types event.");
        let e = ExplorerEntity {
            id: uid(8675309_u32.to_le_bytes()),
            nonce: Bytes32::default(),
            // TOOD: Finish
            time: None,
            hex: Some(Bytes::from("hello world!")),
            sig: Bytes64::default(),
            bytes: Bytes64::default(),
        };

        e.save();
    }

    fn fuel_indexer_trigger_enum(
        _first: AnotherSimpleEnum,
        _second: NestedEnum,
        _third: AnotherSimpleEnum,
    ) {
        info!("fuel_indexer_trigger_enum handling trigger_enum event..");

        let e = ComplexEnumEntity {
            id: uid([1]),
            one: Some(EnumEntity::One.into()),
        };
        e.save();
    }

    fn fuel_indexer_trigger_non_indexable_type(_b: BlockData) {
        info!("fuel_indexer_trigger_non_indexable_type handling trigger_non_indexable_type event.");
        let e = UsesVirtualEntity {
            id: uid([1]),
            name: "hello world".to_string(),
            no_table: VirtualEntity {
                name: Some("virtual".to_string()),
                size: 1,
            }
            .into(),
        };

        e.save();
    }

    fn fuel_indexer_trigger_union_type(_b: BlockData) {
        info!("fuel_indexer_trigger_union_type handling trigger_union_type event.");

        let v = VirtualUnionEntity {
            a: Some(2),
            b: None,
            c: Some(6),
            union_type: Some(UnionType::B.into()),
        };

        let vc = VirtualUnionContainerEntity {
            id: uid([1]),
            union_entity: Some(v.into()),
            union_type: UnionType::B.into(),
        };

        vc.save();

        let e = IndexableUnionEntity {
            id: uid([1]),
            a: Some(5),
            b: Some(10),
            c: None,
            union_type: Some(UnionType::A.into()),
        };

        e.save();
    }

    fn fuel_indexer_test_trigger_list(_block: BlockData) {
        info!("fuel_indexer_test_trigger_list handling trigger_list event.");

        let list_fk1 = ListFKType {
            id: uid([1]),
            value: 1,
        };
        list_fk1.save();

        let list_fk2 = ListFKType {
            id: uid([2]),
            value: 2,
        };
        list_fk2.save();

        let list_fk3 = ListFKType {
            id: uid([3]),
            value: 3,
        };
        list_fk3.save();

        let e = ListTypeEntity {
            id: uid([1]),
            foo_field: "hello world".to_string(),
            required_all: vec![list_fk1.id, list_fk2.id, list_fk3.id],
            optional_inner: vec![
                Some("hello".to_string()),
                None,
                Some("world".to_string()),
                None,
            ],
            optional_outer: Some(vec![1, 2, 3, 4, 5]),
            optional_all: Some(vec![Some(1), None, Some(3), None]),
            virtual_optional_inner: vec![
                Some(
                    VirtualEntity {
                        name: Some("foo".to_string()),
                        size: 1,
                    }
                    .into(),
                ),
                None,
                None,
                Some(
                    VirtualEntity {
                        name: Some("bar".to_string()),
                        size: 2,
                    }
                    .into(),
                ),
            ],
            enum_required_all: vec![EnumEntity::One.into(), EnumEntity::Two.into()],
        };

        e.save();
    }

    fn fuel_indexer_test_trigger_generics(ping: Option<Ping>) {
        info!("fuel_indexer_test_trigger_generics handling trigger_generics event.");

        assert!(ping.is_some());

        let ping = ping.unwrap();
        assert_eq!(ping.id, 8888);
        assert_eq!(ping.value, 8888);
        assert_eq!(
            ping.message,
            SizedAsciiString::<32>::new("aaaasdfsdfasdfsdfaasdfsdfasdfsdf".to_string())
                .unwrap()
        );

        let ping = PingEntity {
            id: uid(ping.id.to_le_bytes()),
            value: ping.value,
            message: ping.message.to_string(),
        };

        ping.save();
    }

    fn trigger_sail_blockdata_test(b: BlockData) {
        info!("Sail test block #{}", b.header.height);
    }

    fn trigger_sail_test(cancel: CancelLimitOrder) {
        info!("trigger_sail_test handling trigger_sail_test event");

        let limit_entity = LimitOrderEntity {
            id: uid(cancel.order.id),
            address: cancel.order.address,
            makerAsset: cancel.order.maker_asset,
            takerAsset: cancel.order.taker_asset,
            makerAmount: cancel.order.maker_amount,
            takerAmount: cancel.order.taker_amount,
            takerIsOwner: cancel.order.taker_is_owner,
            salt: cancel.order.salt,
            ts: cancel.order.ts,
        };

        limit_entity.save();

        let cancel_entity = CancelOrderEntity {
            id: uid(cancel.id),
            limit_order: limit_entity.id.clone(),
        };

        cancel_entity.save();
    }
}

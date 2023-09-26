predicate;

#[allow(dead_code)]
enum Predicate1SimpleEnum {
    VariantOne : (),
    VariantTwo : (),
}

struct Predicate1SimpleStruct {
    field_1: u8,
    field_2: u64,
}

configurable {
    U8: u8 = 8u8,
    BOOL: bool = true,
    STRUCT: Predicate1SimpleStruct = Predicate1SimpleStruct {
field_1: 8,
field_2: 16,
    },
    ENUM: Predicate1SimpleEnum = Predicate1SimpleEnum::VariantOne,
}

fn main(
    u_8: u8,
    switch: bool,
    some_struct: Predicate1SimpleStruct,
    some_enum: Predicate1SimpleEnum,
) -> bool {
    u_8 == U8 && switch == BOOL
}

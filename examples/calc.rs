use std::str::FromStr;

use redis_module::{
    key::RedisKey, redis_module, Context, KeyType, NextArg, RedisError, RedisResult, RedisString,
    RedisValue,
};

fn calc(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    let mut args = args.into_iter();

    let operation = args.next_arg()?;
    let operand_key1 = args.next_arg()?;
    let operand_key2 = args.next_arg()?;
    let result_key = args.next_arg()?;

    let operation = Operation::try_from(operation.as_slice())?;

    let operand1: i64 = read_operand(ctx, &operand_key1)?;
    let operand2: i64 = read_operand(ctx, &operand_key2)?;
    let result = match operation {
        Operation::Add => operand1 + operand2,
        Operation::Sub => operand1 - operand2,
        Operation::Mul => operand1 * operand2,
        Operation::Div => operand1 / operand2,
    };
    let result_key = ctx.open_key_writable(&result_key);
    result_key
        .as_string_dma()?
        .write(result.to_string().as_bytes())?;

    Ok(RedisValue::SimpleStringStatic("OK"))
}

fn check_key_is_string(key: &RedisKey, key_name: &RedisString) -> Result<(), RedisError> {
    fn key_type_as_str(key_type: &KeyType) -> &str {
        match key_type {
            KeyType::String => "string",
            KeyType::List => "list",
            KeyType::Set => "set",
            KeyType::Hash => "hash",
            KeyType::ZSet => "zset",
            KeyType::Empty => "empty",
            KeyType::Module => "module",
            KeyType::Stream => "stream",
        }
    }

    match key.key_type() {
        KeyType::String => Ok(()),
        t => Err(RedisError::String(format!(
            "The value stored at key `{key_name}` is a {}, not a string",
            key_type_as_str(&t)
        ))),
    }
}

fn read_operand<T: FromStr>(ctx: &Context, key: &RedisString) -> Result<T, RedisError> {
    fn parse_number_from_bytes<T: FromStr>(bytes: &[u8]) -> Option<T> {
        let s = std::str::from_utf8(bytes).ok()?;
        s.parse::<T>().ok()
    }

    let key_handle = ctx.open_key(key);
    check_key_is_string(&key_handle, key)?;
    let bytes = key_handle.read()?.ok_or_else(RedisError::nonexistent_key)?;
    let value = parse_number_from_bytes(bytes).ok_or_else(|| {
        RedisError::String(format!(
            "The value stored at key `{}` is not a valid number",
            key
        ))
    })?;
    Ok(value)
}

enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

impl TryFrom<&[u8]> for Operation {
    type Error = RedisError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"calc.add" => Ok(Operation::Add),
            b"calc.sub" => Ok(Operation::Sub),
            b"calc.mul" => Ok(Operation::Mul),
            b"calc.div" => Ok(Operation::Div),
            _ => match std::str::from_utf8(value) {
                Ok(s) => Err(RedisError::String(format!("Unknown command `{s}`"))),
                Err(_) => Err(RedisError::Str("Unknown command")),
            },
        }
    }
}

//////////////////////////////////////////////////////

redis_module! {
    name: "calc",
    version: 1,
    allocator: (redis_module::alloc::RedisAlloc, redis_module::alloc::RedisAlloc),
    data_types: [],
    commands: [
        ["calc.add", calc, "write", 1, 3, 1, ""],
        ["calc.sub", calc, "write", 1, 3, 1, ""],
        ["calc.mul", calc, "write", 1, 3, 1, ""],
        ["calc.div", calc, "write", 1, 3, 1, ""],
    ],
}

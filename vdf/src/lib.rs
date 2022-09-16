#[derive(Debug)]
pub enum Error {}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Value {
    Object(Object),
    String(String),
    Integer(u32),
}

#[derive(Debug)]
pub struct Object {
    values: Vec<(String, Value)>,
}

impl Object {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn set_value(&mut self, key: String, value: Value) {
        self.values.push((key, value));
    }

    pub fn value(&mut self, key: &str) -> Option<&Value> {
        for value in self.values.iter() {
            if value.0 == key {
                return Some(&value.1);
            }
        }

        None
    }

    pub fn value_mut(&mut self, key: &str) -> Option<&mut Value> {
        for value in self.values.iter_mut() {
            if value.0 == key {
                return Some(&mut value.1);
            }
        }

        None
    }
}

pub fn parse_string(bytes: &[u8], offset: &mut usize) -> Result<String> {
    let start = *offset;

    loop {
        let b = bytes[*offset];

        if b == 0x00 {
            break;
        }

        *offset += 1;
    }

    // Terminator Byte
    *offset += 1;

    let s = std::str::from_utf8(&bytes[start..*offset - 1]).unwrap();
    Ok(s.to_string())
}

pub fn parse_int(bytes: &[u8], offset: &mut usize) -> Result<u32> {
    let result =
        u32::from_le_bytes(bytes[*offset..*offset + 4].try_into().unwrap());
    *offset += 4;

    Ok(result)
}

pub fn parse_object(bytes: &[u8], offset: &mut usize) -> Result<Object> {
    let mut obj = Object::new();

    loop {
        if *offset >= bytes.len() {
            // TODO(patrik): This is should be an error, i think
            break;
        }

        let typ = bytes[*offset];
        *offset += 1;

        // Object end marker
        if typ == 0x08 {
            break;
        }

        let name = parse_string(bytes, offset)?;

        match typ {
            // Object
            0x00 => {
                let o = parse_object(bytes, offset)?;
                obj.set_value(name, Value::Object(o));
            }

            0x01 => {
                let s = parse_string(bytes, offset)?;
                obj.set_value(name, Value::String(s));
            }

            0x02 => {
                let value = parse_int(bytes, offset)?;
                obj.set_value(name, Value::Integer(value));
            }

            _ => unimplemented!("Typ: {:#x}", typ),
        }
    }

    Ok(obj)
}

pub fn parse(bytes: &[u8]) -> Result<Object> {
    let mut offset = 0;
    parse_object(bytes, &mut offset)
}

fn write_string(buffer: &mut Vec<u8>, s: &String) -> Result<()> {
    buffer.extend_from_slice(s.as_bytes());
    buffer.push(b'\0');
    Ok(())
}

fn write_integer(buffer: &mut Vec<u8>, i: u32) -> Result<()> {
    buffer.extend_from_slice(&i.to_le_bytes());
    Ok(())
}

fn write_object(buffer: &mut Vec<u8>, obj: &Object) -> Result<()> {
    for value in obj.values.iter() {
        match &value.1 {
            Value::Object(o) => {
                buffer.push(0x00);
                write_string(buffer, &value.0)?;
                write_object(buffer, o)?;
            }

            Value::String(s) => {
                buffer.push(0x01);
                write_string(buffer, &value.0)?;
                write_string(buffer, s)?;
            }

            Value::Integer(i) => {
                buffer.push(0x02);
                write_string(buffer, &value.0)?;
                write_integer(buffer, *i)?;
            }
        }
    }

    buffer.push(0x08);

    Ok(())
}

pub fn write(obj: &Object) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    write_object(&mut buffer, obj)?;

    Ok(buffer)
}

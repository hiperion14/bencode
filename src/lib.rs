use std::ops::{Index, IndexMut};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum BeeValue {
    Null,
    String(String),
    Raw(Vec<u8>),
    Integer(i32),
    List(Vec<BeeValue>),
    Dictionary(std::collections::HashMap<String, BeeValue>),
}

impl Index<&str> for BeeValue {
    type Output = BeeValue;

    fn index(&self, key: &str) -> &Self::Output {
        if let BeeValue::Dictionary(map) = self {
            map.get(key).unwrap_or(&BeeValue::Null)
        } else {
            &BeeValue::Null
        }
    }
}

impl IndexMut<&str> for BeeValue {
    fn index_mut(&mut self, key: &str) -> &mut Self::Output {
        if let BeeValue::Dictionary(map) = self {
            map.entry(key.to_string()).or_insert(BeeValue::Null)
        } else {
            panic!("Cannot index non-Object JSON value");
        }
    }
}

impl Index<usize> for BeeValue {
    type Output = BeeValue;

    fn index(&self, index: usize) -> &Self::Output {
        if let BeeValue::List(vec) = self {
            vec.get(index).unwrap_or(&BeeValue::Null)
        } else {
            &BeeValue::Null
        }
    }
}

impl IndexMut<usize> for BeeValue {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if let BeeValue::List(vec) = self {
            if index >= vec.len() {
                vec.resize(index + 1, BeeValue::Null);
            }
            &mut vec[index]
        } else {
            panic!("Cannot index non-Array JSON value");
        }
    }
}

impl BeeValue {
    pub fn from_bytes(bytes: &Vec<u8>) -> BeeValue {
        let (value, _size) = Self::dfs(bytes, 0);
        return value;
    }

    fn dfs(bytes: &Vec<u8>, index: usize) -> (BeeValue, usize) {
        //String
        if bytes[index].is_ascii_digit() {
            let position = bytes.iter().skip(index).position(|&r| r == 0x3A).unwrap()+index;
            let length = String::from_utf8(bytes[index..position].to_vec()).unwrap().parse::<usize>().unwrap();
            let text = String::from_utf8(bytes[position+1..position+1+length].to_vec());
            if text.is_err() {
                return (BeeValue::Raw(bytes[position+1..position+1+length].to_vec()), position+1+length);
            }
            return (BeeValue::String(text.unwrap()), position+1+length);
        }
        //Integer
        if bytes[index] == 0x69 {
            let position = bytes.iter().skip(index).position(|&r| r == 0x65).unwrap() + index;
            let number = String::from_utf8(bytes[index+1..position].to_vec()).unwrap().parse::<i32>().unwrap();
            return (BeeValue::Integer(number), position+1);
        }
        //List
        if bytes[index] == 0x6C {
            let mut local_index = index+1;
            let mut list: Vec<BeeValue> = vec![];
            while bytes[local_index] != 0x65 {
                let value;
                (value, local_index) = Self::dfs(bytes, local_index);
                list.push(value);
            }
            return (BeeValue::List(list), local_index + 1);
        }
        //Dict
        if bytes[index] == 0x64 {
            let mut local_index = index+1;
            let mut dict = BeeValue::Dictionary(HashMap::new());
            let mut key: String = String::from("");
            let mut i = 0;
            while bytes[local_index] != 0x65 {

                let value;
                (value, local_index) = Self::dfs(bytes, local_index);

                if i % 2 == 1 {
                    dict[key.as_str()] = value;
                } else {
                    key = value.get_string().unwrap();
                }
                i+=1;
            }
            return (dict, local_index + 1);
        }
        return (BeeValue::Null, index);
    }

    pub fn get_string(&self) -> Option<String> {
        if let BeeValue::String(s) = self {
            Some(s.to_string())
        } else {
            None
        }
    }

    pub fn get_int(&self) -> Option<i32> {
        if let BeeValue::Integer(i) = self {
            Some(i.to_owned())
        } else {
            None
        }
    }
    
    pub fn get_raw(&self) -> Option<Vec<u8>> {
        if let BeeValue::Raw(v) = self {
            Some(v.to_vec())
        } else {
            None
        }
    }
}
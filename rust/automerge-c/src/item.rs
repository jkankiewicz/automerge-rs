use automerge as am;

use std::any::type_name;
use std::borrow::Cow;
use std::cell::{RefCell, UnsafeCell};
use std::rc::Rc;

use crate::actor_id::AMactorId;
use crate::byte_span::{to_str, AMbyteSpan};
use crate::change::AMchange;
use crate::doc::AMdoc;
use crate::index::{AMidxType, AMindex};
use crate::obj::AMobjId;
use crate::result::{to_result, AMresult};
use crate::sync::{AMsyncHave, AMsyncMessage, AMsyncState};

/// \struct AMunknownValue
/// \installed_headerfile
/// \brief A value (typically for a `set` operation) whose type is unknown.
#[derive(Default, Eq, PartialEq)]
#[repr(C)]
pub struct AMunknownValue {
    /// The value's raw bytes.
    bytes: AMbyteSpan,
    /// The value's encoded type identifier.
    type_code: u8,
}

pub enum Value {
    ActorId(am::ActorId, UnsafeCell<Option<AMactorId>>),
    Change(Box<am::Change>, UnsafeCell<Option<AMchange>>),
    ChangeHash(am::ChangeHash),
    Doc(RefCell<AMdoc>),
    SyncHave(AMsyncHave),
    SyncMessage(AMsyncMessage),
    SyncState(RefCell<AMsyncState>),
    Value(am::Value<'static>),
}

impl Value {
    pub fn try_into_bytes(&self) -> Result<AMbyteSpan, am::AutomergeError> {
        use am::AutomergeError::InvalidValueType;
        use am::ScalarValue::*;
        use am::Value::*;

        if let Self::Value(Scalar(scalar)) = &self {
            if let Bytes(vector) = scalar.as_ref() {
                return Ok(vector.as_slice().into());
            }
        }
        Err(InvalidValueType {
            expected: type_name::<AMbyteSpan>().to_string(),
            unexpected: type_name::<self::Value>().to_string(),
        })
    }

    pub fn try_into_change_hash(&self) -> Result<AMbyteSpan, am::AutomergeError> {
        use am::AutomergeError::InvalidValueType;

        if let Self::ChangeHash(change_hash) = &self {
            return Ok(change_hash.into());
        }
        Err(InvalidValueType {
            expected: type_name::<AMbyteSpan>().to_string(),
            unexpected: type_name::<self::Value>().to_string(),
        })
    }

    pub fn try_into_counter(&self) -> Result<i64, am::AutomergeError> {
        use am::AutomergeError::InvalidValueType;
        use am::ScalarValue::*;
        use am::Value::*;

        if let Self::Value(Scalar(scalar)) = &self {
            if let Counter(counter) = scalar.as_ref() {
                return Ok(counter.into());
            }
        }
        Err(InvalidValueType {
            expected: type_name::<i64>().to_string(),
            unexpected: type_name::<self::Value>().to_string(),
        })
    }

    pub fn try_into_int(&self) -> Result<i64, am::AutomergeError> {
        use am::AutomergeError::InvalidValueType;
        use am::ScalarValue::*;
        use am::Value::*;

        if let Self::Value(Scalar(scalar)) = &self {
            if let Int(int) = scalar.as_ref() {
                return Ok(*int);
            }
        }
        Err(InvalidValueType {
            expected: type_name::<i64>().to_string(),
            unexpected: type_name::<self::Value>().to_string(),
        })
    }

    pub fn try_into_str(&self) -> Result<AMbyteSpan, am::AutomergeError> {
        use am::AutomergeError::InvalidValueType;
        use am::ScalarValue::*;
        use am::Value::*;

        if let Self::Value(Scalar(scalar)) = &self {
            if let Str(smol_str) = scalar.as_ref() {
                return Ok(smol_str.into());
            }
        }
        Err(InvalidValueType {
            expected: type_name::<AMbyteSpan>().to_string(),
            unexpected: type_name::<self::Value>().to_string(),
        })
    }

    pub fn try_into_timestamp(&self) -> Result<i64, am::AutomergeError> {
        use am::AutomergeError::InvalidValueType;
        use am::ScalarValue::*;
        use am::Value::*;

        if let Self::Value(Scalar(scalar)) = &self {
            if let Timestamp(timestamp) = scalar.as_ref() {
                return Ok(*timestamp);
            }
        }
        Err(InvalidValueType {
            expected: type_name::<i64>().to_string(),
            unexpected: type_name::<self::Value>().to_string(),
        })
    }
}

impl From<am::ActorId> for Value {
    fn from(actor_id: am::ActorId) -> Self {
        Self::ActorId(actor_id, Default::default())
    }
}

impl From<am::AutoCommit> for Value {
    fn from(auto_commit: am::AutoCommit) -> Self {
        Self::Doc(RefCell::new(AMdoc::new(auto_commit)))
    }
}

impl From<am::Change> for Value {
    fn from(change: am::Change) -> Self {
        Self::Change(Box::new(change), Default::default())
    }
}

impl From<am::ChangeHash> for Value {
    fn from(change_hash: am::ChangeHash) -> Self {
        Self::ChangeHash(change_hash)
    }
}

impl From<am::sync::Have> for Value {
    fn from(have: am::sync::Have) -> Self {
        Self::SyncHave(AMsyncHave::new(have))
    }
}

impl From<am::sync::Message> for Value {
    fn from(message: am::sync::Message) -> Self {
        Self::SyncMessage(AMsyncMessage::new(message))
    }
}

impl From<am::sync::State> for Value {
    fn from(state: am::sync::State) -> Self {
        Self::SyncState(RefCell::new(AMsyncState::new(state)))
    }
}

impl From<am::Value<'static>> for Value {
    fn from(value: am::Value<'static>) -> Self {
        Self::Value(value)
    }
}

impl From<String> for Value {
    fn from(string: String) -> Self {
        Self::Value(am::Value::Scalar(Cow::Owned(am::ScalarValue::Str(
            string.into(),
        ))))
    }
}

impl<'a> TryFrom<&'a Value> for &'a am::Change {
    type Error = am::AutomergeError;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        use self::Value::*;
        use am::AutomergeError::InvalidValueType;

        match value {
            Change(change, _) => Ok(change),
            _ => Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<self::Value>().to_string(),
            }),
        }
    }
}

impl<'a> TryFrom<&'a Value> for &'a am::ChangeHash {
    type Error = am::AutomergeError;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        use self::Value::*;
        use am::AutomergeError::InvalidValueType;

        match value {
            ChangeHash(change_hash) => Ok(change_hash),
            _ => Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<self::Value>().to_string(),
            }),
        }
    }
}

impl<'a> TryFrom<&'a Value> for &'a am::ScalarValue {
    type Error = am::AutomergeError;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        use self::Value::*;
        use am::AutomergeError::InvalidValueType;
        use am::Value::*;

        if let Value(Scalar(scalar)) = value {
            return Ok(scalar.as_ref());
        }
        Err(InvalidValueType {
            expected: type_name::<Self>().to_string(),
            unexpected: type_name::<self::Value>().to_string(),
        })
    }
}

impl<'a> TryFrom<&'a Value> for &'a AMactorId {
    type Error = am::AutomergeError;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        use self::Value::*;
        use am::AutomergeError::InvalidValueType;

        match value {
            ActorId(actor_id, c_actor_id) => unsafe {
                Ok((*c_actor_id.get()).get_or_insert(AMactorId::new(actor_id)))
            },
            _ => Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<self::Value>().to_string(),
            }),
        }
    }
}

impl<'a> TryFrom<&'a mut Value> for &'a mut AMchange {
    type Error = am::AutomergeError;

    fn try_from(value: &'a mut Value) -> Result<Self, Self::Error> {
        use self::Value::*;
        use am::AutomergeError::InvalidValueType;

        match value {
            Change(change, c_change) => unsafe {
                Ok((*c_change.get()).get_or_insert(AMchange::new(change)))
            },
            _ => Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<self::Value>().to_string(),
            }),
        }
    }
}

impl<'a> TryFrom<&'a mut Value> for &'a mut AMdoc {
    type Error = am::AutomergeError;

    fn try_from(value: &'a mut Value) -> Result<Self, Self::Error> {
        use self::Value::*;
        use am::AutomergeError::InvalidValueType;

        match value {
            Doc(doc) => Ok(doc.get_mut()),
            _ => Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<self::Value>().to_string(),
            }),
        }
    }
}

impl<'a> TryFrom<&'a Value> for &'a AMsyncHave {
    type Error = am::AutomergeError;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        use self::Value::*;
        use am::AutomergeError::InvalidValueType;

        match value {
            SyncHave(sync_have) => Ok(sync_have),
            _ => Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<self::Value>().to_string(),
            }),
        }
    }
}

impl<'a> TryFrom<&'a Value> for &'a AMsyncMessage {
    type Error = am::AutomergeError;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        use self::Value::*;
        use am::AutomergeError::InvalidValueType;

        match value {
            SyncMessage(sync_message) => Ok(sync_message),
            _ => Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<self::Value>().to_string(),
            }),
        }
    }
}

impl<'a> TryFrom<&'a mut Value> for &'a mut AMsyncState {
    type Error = am::AutomergeError;

    fn try_from(value: &'a mut Value) -> Result<Self, Self::Error> {
        use self::Value::*;
        use am::AutomergeError::InvalidValueType;

        match value {
            SyncState(sync_state) => Ok(sync_state.get_mut()),
            _ => Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<self::Value>().to_string(),
            }),
        }
    }
}

impl TryFrom<&Value> for bool {
    type Error = am::AutomergeError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        use self::Value::*;
        use am::AutomergeError::InvalidValueType;
        use am::ScalarValue::*;
        use am::Value::*;

        if let Value(Scalar(scalar)) = value {
            if let Boolean(boolean) = scalar.as_ref() {
                return Ok(*boolean);
            }
        }
        Err(InvalidValueType {
            expected: type_name::<Self>().to_string(),
            unexpected: type_name::<self::Value>().to_string(),
        })
    }
}

impl TryFrom<&Value> for f64 {
    type Error = am::AutomergeError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        use self::Value::*;
        use am::AutomergeError::InvalidValueType;
        use am::ScalarValue::*;
        use am::Value::*;

        if let Value(Scalar(scalar)) = value {
            if let F64(float) = scalar.as_ref() {
                return Ok(*float);
            }
        }
        Err(InvalidValueType {
            expected: type_name::<Self>().to_string(),
            unexpected: type_name::<self::Value>().to_string(),
        })
    }
}

impl TryFrom<&Value> for u64 {
    type Error = am::AutomergeError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        use self::Value::*;
        use am::AutomergeError::InvalidValueType;
        use am::ScalarValue::*;
        use am::Value::*;

        if let Value(Scalar(scalar)) = value {
            if let Uint(uint) = scalar.as_ref() {
                return Ok(*uint);
            }
        }
        Err(InvalidValueType {
            expected: type_name::<Self>().to_string(),
            unexpected: type_name::<self::Value>().to_string(),
        })
    }
}

impl TryFrom<&Value> for AMunknownValue {
    type Error = am::AutomergeError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        use self::Value::*;
        use am::AutomergeError::InvalidValueType;
        use am::ScalarValue::*;
        use am::Value::*;

        if let Value(Scalar(scalar)) = value {
            if let Unknown { bytes, type_code } = scalar.as_ref() {
                return Ok(Self {
                    bytes: bytes.as_slice().into(),
                    type_code: *type_code,
                });
            }
        }
        Err(InvalidValueType {
            expected: type_name::<Self>().to_string(),
            unexpected: type_name::<self::Value>().to_string(),
        })
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use self::Value::*;

        match (self, other) {
            (ActorId(lhs, _), ActorId(rhs, _)) => *lhs == *rhs,
            (Change(lhs, _), Change(rhs, _)) => lhs == rhs,
            (ChangeHash(lhs), ChangeHash(rhs)) => lhs == rhs,
            (Doc(lhs), Doc(rhs)) => lhs.as_ptr() == rhs.as_ptr(),
            (SyncMessage(lhs), SyncMessage(rhs)) => *lhs == *rhs,
            (SyncState(lhs), SyncState(rhs)) => *lhs == *rhs,
            (Value(lhs), Value(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

#[derive(Default)]
pub struct Item {
    /// The item's index.
    index: Option<AMindex>,
    /// The item's identifier.
    obj_id: Option<AMobjId>,
    /// The item's value.
    value: Option<Value>,
}

impl Item {
    pub fn try_into_bytes(&self) -> Result<AMbyteSpan, am::AutomergeError> {
        use am::AutomergeError::InvalidValueType;

        if let Some(value) = &self.value {
            return value.try_into_bytes();
        }
        Err(InvalidValueType {
            expected: type_name::<AMbyteSpan>().to_string(),
            unexpected: type_name::<Option<Value>>().to_string(),
        })
    }

    pub fn try_into_change_hash(&self) -> Result<AMbyteSpan, am::AutomergeError> {
        use am::AutomergeError::InvalidValueType;

        if let Some(value) = &self.value {
            return value.try_into_change_hash();
        }
        Err(InvalidValueType {
            expected: type_name::<AMbyteSpan>().to_string(),
            unexpected: type_name::<Option<Value>>().to_string(),
        })
    }

    pub fn try_into_counter(&self) -> Result<i64, am::AutomergeError> {
        use am::AutomergeError::InvalidValueType;

        if let Some(value) = &self.value {
            return value.try_into_counter();
        }
        Err(InvalidValueType {
            expected: type_name::<i64>().to_string(),
            unexpected: type_name::<Option<Value>>().to_string(),
        })
    }

    pub fn try_into_int(&self) -> Result<i64, am::AutomergeError> {
        use am::AutomergeError::InvalidValueType;

        if let Some(value) = &self.value {
            return value.try_into_int();
        }
        Err(InvalidValueType {
            expected: type_name::<i64>().to_string(),
            unexpected: type_name::<Option<Value>>().to_string(),
        })
    }

    pub fn try_into_str(&self) -> Result<AMbyteSpan, am::AutomergeError> {
        use am::AutomergeError::InvalidValueType;

        if let Some(value) = &self.value {
            return value.try_into_str();
        }
        Err(InvalidValueType {
            expected: type_name::<AMbyteSpan>().to_string(),
            unexpected: type_name::<Option<Value>>().to_string(),
        })
    }

    pub fn try_into_timestamp(&self) -> Result<i64, am::AutomergeError> {
        use am::AutomergeError::InvalidValueType;

        if let Some(value) = &self.value {
            return value.try_into_timestamp();
        }
        Err(InvalidValueType {
            expected: type_name::<i64>().to_string(),
            unexpected: type_name::<Option<Value>>().to_string(),
        })
    }
}

impl From<am::ActorId> for Item {
    fn from(actor_id: am::ActorId) -> Self {
        Value::from(actor_id).into()
    }
}

impl From<am::AutoCommit> for Item {
    fn from(auto_commit: am::AutoCommit) -> Self {
        Value::from(auto_commit).into()
    }
}

impl From<am::Change> for Item {
    fn from(change: am::Change) -> Self {
        Value::from(change).into()
    }
}

impl From<am::ChangeHash> for Item {
    fn from(change_hash: am::ChangeHash) -> Self {
        Value::from(change_hash).into()
    }
}

impl From<am::ObjId> for Item {
    fn from(obj_id: am::ObjId) -> Self {
        Self {
            index: None,
            obj_id: Some(AMobjId::new(obj_id)),
            value: None,
        }
    }
}

impl From<am::sync::Have> for Item {
    fn from(have: am::sync::Have) -> Self {
        Value::from(have).into()
    }
}

impl From<am::sync::Message> for Item {
    fn from(message: am::sync::Message) -> Self {
        Value::from(message).into()
    }
}

impl From<am::sync::State> for Item {
    fn from(state: am::sync::State) -> Self {
        Value::from(state).into()
    }
}

impl From<am::Value<'static>> for Item {
    fn from(value: am::Value<'static>) -> Self {
        Value::from(value).into()
    }
}

impl From<String> for Item {
    fn from(string: String) -> Self {
        Value::from(string).into()
    }
}

impl From<Value> for Item {
    fn from(value: Value) -> Self {
        Self {
            index: None,
            obj_id: None,
            value: Some(value),
        }
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.obj_id == other.obj_id && self.value == other.value
    }
}

impl<'a> TryFrom<&'a Item> for &'a am::Change {
    type Error = am::AutomergeError;

    fn try_from(item: &'a Item) -> Result<Self, Self::Error> {
        use am::AutomergeError::InvalidValueType;

        if let Some(value) = &item.value {
            value.try_into()
        } else {
            Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<Option<Value>>().to_string(),
            })
        }
    }
}

impl<'a> TryFrom<&'a Item> for &'a am::ChangeHash {
    type Error = am::AutomergeError;

    fn try_from(item: &'a Item) -> Result<Self, Self::Error> {
        use am::AutomergeError::InvalidValueType;

        if let Some(value) = &item.value {
            value.try_into()
        } else {
            Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<Option<Value>>().to_string(),
            })
        }
    }
}

impl<'a> TryFrom<&'a Item> for &'a am::ScalarValue {
    type Error = am::AutomergeError;

    fn try_from(item: &'a Item) -> Result<Self, Self::Error> {
        use am::AutomergeError::InvalidValueType;

        if let Some(value) = &item.value {
            value.try_into()
        } else {
            Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<Option<Value>>().to_string(),
            })
        }
    }
}

impl<'a> TryFrom<&'a Item> for &'a AMactorId {
    type Error = am::AutomergeError;

    fn try_from(item: &'a Item) -> Result<Self, Self::Error> {
        use am::AutomergeError::InvalidValueType;

        if let Some(value) = &item.value {
            value.try_into()
        } else {
            Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<Option<Value>>().to_string(),
            })
        }
    }
}

impl<'a> TryFrom<&'a mut Item> for &'a mut AMchange {
    type Error = am::AutomergeError;

    fn try_from(item: &'a mut Item) -> Result<Self, Self::Error> {
        use am::AutomergeError::InvalidValueType;

        if let Some(value) = &mut item.value {
            value.try_into()
        } else {
            Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<Option<Value>>().to_string(),
            })
        }
    }
}

impl<'a> TryFrom<&'a mut Item> for &'a mut AMdoc {
    type Error = am::AutomergeError;

    fn try_from(item: &'a mut Item) -> Result<Self, Self::Error> {
        use am::AutomergeError::InvalidValueType;

        if let Some(value) = &mut item.value {
            value.try_into()
        } else {
            Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<Option<Value>>().to_string(),
            })
        }
    }
}

impl From<&Item> for AMidxType {
    fn from(item: &Item) -> Self {
        if let Some(index) = &item.index {
            return index.into();
        }
        Default::default()
    }
}

impl<'a> TryFrom<&'a Item> for &'a AMsyncHave {
    type Error = am::AutomergeError;

    fn try_from(item: &'a Item) -> Result<Self, Self::Error> {
        use am::AutomergeError::InvalidValueType;

        if let Some(value) = &item.value {
            value.try_into()
        } else {
            Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<Option<Value>>().to_string(),
            })
        }
    }
}

impl<'a> TryFrom<&'a Item> for &'a AMsyncMessage {
    type Error = am::AutomergeError;

    fn try_from(item: &'a Item) -> Result<Self, Self::Error> {
        use am::AutomergeError::InvalidValueType;

        if let Some(value) = &item.value {
            value.try_into()
        } else {
            Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<Option<Value>>().to_string(),
            })
        }
    }
}

impl<'a> TryFrom<&'a mut Item> for &'a mut AMsyncState {
    type Error = am::AutomergeError;

    fn try_from(item: &'a mut Item) -> Result<Self, Self::Error> {
        use am::AutomergeError::InvalidValueType;

        if let Some(value) = &mut item.value {
            value.try_into()
        } else {
            Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<Option<Value>>().to_string(),
            })
        }
    }
}

impl TryFrom<&Item> for bool {
    type Error = am::AutomergeError;

    fn try_from(item: &Item) -> Result<Self, Self::Error> {
        use am::AutomergeError::InvalidValueType;

        if let Some(value) = &item.value {
            value.try_into()
        } else {
            Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<Option<Value>>().to_string(),
            })
        }
    }
}

impl TryFrom<&Item> for f64 {
    type Error = am::AutomergeError;

    fn try_from(item: &Item) -> Result<Self, Self::Error> {
        use am::AutomergeError::InvalidValueType;

        if let Some(value) = &item.value {
            value.try_into()
        } else {
            Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<Option<Value>>().to_string(),
            })
        }
    }
}

impl TryFrom<&Item> for u64 {
    type Error = am::AutomergeError;

    fn try_from(item: &Item) -> Result<Self, Self::Error> {
        use am::AutomergeError::InvalidValueType;

        if let Some(value) = &item.value {
            value.try_into()
        } else {
            Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<Option<Value>>().to_string(),
            })
        }
    }
}

impl TryFrom<&Item> for AMunknownValue {
    type Error = am::AutomergeError;

    fn try_from(item: &Item) -> Result<Self, Self::Error> {
        use am::AutomergeError::InvalidValueType;

        if let Some(value) = &item.value {
            value.try_into()
        } else {
            Err(InvalidValueType {
                expected: type_name::<Self>().to_string(),
                unexpected: type_name::<Option<Value>>().to_string(),
            })
        }
    }
}

impl TryFrom<&Item> for (am::Value<'static>, am::ObjId) {
    type Error = am::AutomergeError;

    fn try_from(item: &Item) -> Result<Self, Self::Error> {
        use self::Value::*;
        use am::AutomergeError::InvalidObjId;
        use am::AutomergeError::InvalidValueType;

        let expected = type_name::<am::Value>().to_string();
        match (&item.obj_id, &item.value) {
            (None, None) | (None, Some(_)) => Err(InvalidObjId("".to_string())),
            (Some(_), None) => Err(InvalidValueType {
                expected,
                unexpected: type_name::<Option<am::Value>>().to_string(),
            }),
            (Some(obj_id), Some(value)) => match value {
                ActorId(_, _) => Err(InvalidValueType {
                    expected,
                    unexpected: type_name::<AMactorId>().to_string(),
                }),
                ChangeHash(_) => Err(InvalidValueType {
                    expected,
                    unexpected: type_name::<am::ChangeHash>().to_string(),
                }),
                Change(_, _) => Err(InvalidValueType {
                    expected,
                    unexpected: type_name::<AMchange>().to_string(),
                }),
                Doc(_) => Err(InvalidValueType {
                    expected,
                    unexpected: type_name::<AMdoc>().to_string(),
                }),
                SyncHave(_) => Err(InvalidValueType {
                    expected,
                    unexpected: type_name::<AMsyncHave>().to_string(),
                }),
                SyncMessage(_) => Err(InvalidValueType {
                    expected,
                    unexpected: type_name::<AMsyncMessage>().to_string(),
                }),
                SyncState(_) => Err(InvalidValueType {
                    expected,
                    unexpected: type_name::<AMsyncState>().to_string(),
                }),
                Value(v) => Ok((v.clone(), obj_id.as_ref().clone())),
            },
        }
    }
}

/// \struct AMitem
/// \installed_headerfile
/// \brief An item within a result.
#[derive(Clone)]
pub struct AMitem(Rc<Item>);

impl AMitem {
    pub fn exact(obj_id: am::ObjId, value: Value) -> Self {
        Self(Rc::new(Item {
            index: None,
            obj_id: Some(AMobjId::new(obj_id)),
            value: Some(value),
        }))
    }

    pub fn indexed(index: AMindex, obj_id: am::ObjId, value: Value) -> Self {
        Self(Rc::new(Item {
            index: Some(index),
            obj_id: Some(AMobjId::new(obj_id)),
            value: Some(value),
        }))
    }
}

impl AsRef<Item> for AMitem {
    fn as_ref(&self) -> &Item {
        self.0.as_ref()
    }
}

impl Default for AMitem {
    fn default() -> Self {
        Self(Rc::new(Item {
            index: None,
            obj_id: None,
            value: None,
        }))
    }
}

impl From<am::ActorId> for AMitem {
    fn from(actor_id: am::ActorId) -> Self {
        Value::from(actor_id).into()
    }
}

impl From<am::AutoCommit> for AMitem {
    fn from(auto_commit: am::AutoCommit) -> Self {
        Value::from(auto_commit).into()
    }
}

impl From<am::Change> for AMitem {
    fn from(change: am::Change) -> Self {
        Value::from(change).into()
    }
}

impl From<am::ChangeHash> for AMitem {
    fn from(change_hash: am::ChangeHash) -> Self {
        Value::from(change_hash).into()
    }
}

impl From<am::ObjId> for AMitem {
    fn from(obj_id: am::ObjId) -> Self {
        Self(Rc::new(Item::from(obj_id)))
    }
}

impl From<am::sync::Have> for AMitem {
    fn from(have: am::sync::Have) -> Self {
        Value::from(have).into()
    }
}

impl From<am::sync::Message> for AMitem {
    fn from(message: am::sync::Message) -> Self {
        Value::from(message).into()
    }
}

impl From<am::sync::State> for AMitem {
    fn from(state: am::sync::State) -> Self {
        Value::from(state).into()
    }
}

impl From<am::Value<'static>> for AMitem {
    fn from(value: am::Value<'static>) -> Self {
        Value::from(value).into()
    }
}

impl From<String> for AMitem {
    fn from(string: String) -> Self {
        Value::from(string).into()
    }
}

impl From<Value> for AMitem {
    fn from(value: Value) -> Self {
        Self(Rc::new(Item::from(value)))
    }
}

impl PartialEq for AMitem {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<'a> TryFrom<&'a AMitem> for &'a am::Change {
    type Error = am::AutomergeError;

    fn try_from(item: &'a AMitem) -> Result<Self, Self::Error> {
        item.as_ref().try_into()
    }
}

impl<'a> TryFrom<&'a AMitem> for &'a am::ChangeHash {
    type Error = am::AutomergeError;

    fn try_from(item: &'a AMitem) -> Result<Self, Self::Error> {
        item.as_ref().try_into()
    }
}

impl<'a> TryFrom<&'a AMitem> for &'a am::ScalarValue {
    type Error = am::AutomergeError;

    fn try_from(item: &'a AMitem) -> Result<Self, Self::Error> {
        item.as_ref().try_into()
    }
}

impl<'a> TryFrom<&'a AMitem> for &'a AMactorId {
    type Error = am::AutomergeError;

    fn try_from(item: &'a AMitem) -> Result<Self, Self::Error> {
        item.as_ref().try_into()
    }
}

impl<'a> TryFrom<&'a mut AMitem> for &'a mut AMchange {
    type Error = am::AutomergeError;

    fn try_from(item: &'a mut AMitem) -> Result<Self, Self::Error> {
        if let Some(item) = Rc::get_mut(&mut item.0) {
            item.try_into()
        } else {
            Err(Self::Error::Fail)
        }
    }
}

impl<'a> TryFrom<&'a mut AMitem> for &'a mut AMdoc {
    type Error = am::AutomergeError;

    fn try_from(item: &'a mut AMitem) -> Result<Self, Self::Error> {
        if let Some(item) = Rc::get_mut(&mut item.0) {
            item.try_into()
        } else {
            Err(Self::Error::Fail)
        }
    }
}

impl<'a> TryFrom<&'a AMitem> for &'a AMsyncHave {
    type Error = am::AutomergeError;

    fn try_from(item: &'a AMitem) -> Result<Self, Self::Error> {
        item.as_ref().try_into()
    }
}

impl<'a> TryFrom<&'a AMitem> for &'a AMsyncMessage {
    type Error = am::AutomergeError;

    fn try_from(item: &'a AMitem) -> Result<Self, Self::Error> {
        item.as_ref().try_into()
    }
}

impl<'a> TryFrom<&'a mut AMitem> for &'a mut AMsyncState {
    type Error = am::AutomergeError;

    fn try_from(item: &'a mut AMitem) -> Result<Self, Self::Error> {
        if let Some(item) = Rc::get_mut(&mut item.0) {
            item.try_into()
        } else {
            Err(Self::Error::Fail)
        }
    }
}

impl TryFrom<&AMitem> for bool {
    type Error = am::AutomergeError;

    fn try_from(item: &AMitem) -> Result<Self, Self::Error> {
        item.as_ref().try_into()
    }
}

impl TryFrom<&AMitem> for f64 {
    type Error = am::AutomergeError;

    fn try_from(item: &AMitem) -> Result<Self, Self::Error> {
        item.as_ref().try_into()
    }
}

impl TryFrom<&AMitem> for u64 {
    type Error = am::AutomergeError;

    fn try_from(item: &AMitem) -> Result<Self, Self::Error> {
        item.as_ref().try_into()
    }
}

impl TryFrom<&AMitem> for AMunknownValue {
    type Error = am::AutomergeError;

    fn try_from(item: &AMitem) -> Result<Self, Self::Error> {
        item.as_ref().try_into()
    }
}

impl TryFrom<&AMitem> for (am::Value<'static>, am::ObjId) {
    type Error = am::AutomergeError;

    fn try_from(item: &AMitem) -> Result<Self, Self::Error> {
        item.as_ref().try_into()
    }
}

/// \ingroup enumerations
/// \enum AMvalType
/// \installed_headerfile
/// \brief The type of an item's value.
#[derive(PartialEq, Eq)]
#[repr(u32)]
pub enum AMvalType {
    /// An actor identifier value.
    ActorId = 1 << 1,
    /// A boolean value.
    Bool = 1 << 2,
    /// A view onto an array of bytes value.
    Bytes = 1 << 3,
    /// A change value.
    Change = 1 << 4,
    /// A change hash value.
    ChangeHash = 1 << 5,
    /// A CRDT counter value.
    Counter = 1 << 6,
    /// The default tag, not a type signifier.
    Default = 0,
    /// A document value.
    Doc = 1 << 7,
    /// A 64-bit float value.
    F64 = 1 << 8,
    /// A 64-bit signed integer value.
    Int = 1 << 9,
    /// A null value.
    Null = 1 << 10,
    /// An object type value.
    ObjType = 1 << 11,
    /// A UTF-8 string view value.
    Str = 1 << 12,
    /// A synchronization have value.
    SyncHave = 1 << 13,
    /// A synchronization message value.
    SyncMessage = 1 << 14,
    /// A synchronization state value.
    SyncState = 1 << 15,
    /// A *nix timestamp (milliseconds) value.
    Timestamp = 1 << 16,
    /// A 64-bit unsigned integer value.
    Uint = 1 << 17,
    /// An unknown type of value.
    Unknown = 1 << 18,
    /// A void.
    Void = 1 << 0,
}

impl Default for AMvalType {
    fn default() -> Self {
        Self::Default
    }
}

impl From<&am::Value<'static>> for AMvalType {
    fn from(value: &am::Value<'static>) -> Self {
        use am::ScalarValue::*;
        use am::Value::*;

        match value {
            Object(_) => Self::ObjType,
            Scalar(scalar) => match scalar.as_ref() {
                Boolean(_) => Self::Bool,
                Bytes(_) => Self::Bytes,
                Counter(_) => Self::Counter,
                F64(_) => Self::F64,
                Int(_) => Self::Int,
                Null => Self::Null,
                Str(_) => Self::Str,
                Timestamp(_) => Self::Timestamp,
                Uint(_) => Self::Uint,
                Unknown { .. } => Self::Unknown,
            },
        }
    }
}

impl From<&Value> for AMvalType {
    fn from(value: &Value) -> Self {
        use self::Value::*;

        match value {
            ActorId(_, _) => Self::ActorId,
            Change(_, _) => Self::Change,
            ChangeHash(_) => Self::ChangeHash,
            Doc(_) => Self::Doc,
            SyncHave(_) => Self::SyncHave,
            SyncMessage(_) => Self::SyncMessage,
            SyncState(_) => Self::SyncState,
            Value(v) => v.into(),
        }
    }
}

impl From<&Item> for AMvalType {
    fn from(item: &Item) -> Self {
        if let Some(value) = &item.value {
            return value.into();
        }
        Self::Void
    }
}

/// \memberof AMitem
/// \brief Tests the equality of two items.
///
/// \param[in] item1 A pointer to an `AMitem` struct.
/// \param[in] item2 A pointer to an `AMitem` struct.
/// \return `true` if \p item1 `==` \p item2 and `false` otherwise.
/// \pre \p item1 `!= NULL`
/// \pre \p item2 `!= NULL`
/// \post `!(`\p item1 `&&` \p item2 `) -> false`
/// \internal
///
/// #Safety
/// item1 must be a valid AMitem pointer
/// item2 must be a valid AMitem pointer
#[no_mangle]
pub unsafe extern "C" fn AMitemEqual(item1: *const AMitem, item2: *const AMitem) -> bool {
    match (item1.as_ref(), item2.as_ref()) {
        (Some(item1), Some(item2)) => *item1 == *item2,
        (None, None) | (None, Some(_)) | (Some(_), None) => false,
    }
}

/// \memberof AMitem
/// \brief Allocates a new item and initializes it from a boolean value.
///
/// \param[in] value A boolean.
/// \return A pointer to an `AMresult` struct with an `AM_VAL_TYPE_BOOL` item.
/// \warning The returned `AMresult` struct pointer must be passed to `AMfree()`
///          in order to avoid a memory leak.
#[no_mangle]
pub unsafe extern "C" fn AMitemFromBool(value: bool) -> *mut AMresult {
    AMresult::item(am::Value::from(value).into()).into()
}

/// \memberof AMitem
/// \brief Allocates a new item and initializes it from an array of bytes value.
///
/// \param[in] src A pointer to an array of bytes.
/// \param[in] count The count of bytes to copy from the array pointed to by
///                  \p src.
/// \return A pointer to an `AMresult` struct with an `AM_VAL_TYPE_BYTES` item.
/// \pre \p src `!= NULL`
/// \pre `sizeof(`\p src `) > 0`
/// \pre \p count `<= sizeof(`\p src `)`
/// \warning The returned `AMresult` struct pointer must be passed to `AMfree()`
///          in order to avoid a memory leak.
/// \internal
///
/// # Safety
/// value.src must be a byte array of length >= value.count
#[no_mangle]
pub unsafe extern "C" fn AMitemFromBytes(src: *const u8, count: usize) -> *mut AMresult {
    let value = std::slice::from_raw_parts(src, count);
    AMresult::item(am::Value::bytes(value.to_vec()).into()).into()
}

/// \memberof AMitem
/// \brief Allocates a new item and initializes it from a change hash value.
///
/// \param[in] value A change hash as an `AMbyteSpan` struct.
/// \return A pointer to an `AMresult` struct with an `AM_VAL_TYPE_CHANGE_HASH` item.
/// \pre \p value.src `!= NULL`
/// \pre `0 <` \p value.count `<= sizeof(`\p value.src `)`
/// \warning The returned `AMresult` struct pointer must be passed to `AMfree()`
///          in order to avoid a memory leak.
/// \internal
///
/// # Safety
/// value.src must be a byte array of length >= value.count
#[no_mangle]
pub unsafe extern "C" fn AMitemFromChangeHash(value: AMbyteSpan) -> *mut AMresult {
    to_result(am::ChangeHash::try_from(&value))
}

/// \memberof AMitem
/// \brief Allocates a new item and initializes it from a CRDT counter value.
///
/// \param[in] value A 64-bit signed integer.
/// \return A pointer to an `AMresult` struct with an `AM_VAL_TYPE_COUNTER` item.
/// \warning The returned `AMresult` struct pointer must be passed to `AMfree()`
///          in order to avoid a memory leak.
#[no_mangle]
pub unsafe extern "C" fn AMitemFromCounter(value: i64) -> *mut AMresult {
    AMresult::item(am::Value::counter(value).into()).into()
}

/// \memberof AMitem
/// \brief Allocates a new item and initializes it from a float value.
///
/// \param[in] value A 64-bit float.
/// \return A pointer to an `AMresult` struct with an `AM_VAL_TYPE_F64` item.
/// \warning The returned `AMresult` struct pointer must be passed to `AMfree()`
///          in order to avoid a memory leak.
#[no_mangle]
pub unsafe extern "C" fn AMitemFromF64(value: f64) -> *mut AMresult {
    AMresult::item(am::Value::f64(value).into()).into()
}

/// \memberof AMitem
/// \brief Allocates a new item and initializes it from a signed integer value.
///
/// \param[in] value A 64-bit signed integer.
/// \return A pointer to an `AMresult` struct with an `AM_VAL_TYPE_INT` item.
/// \warning The returned `AMresult` struct pointer must be passed to `AMfree()`
///          in order to avoid a memory leak.
#[no_mangle]
pub unsafe extern "C" fn AMitemFromInt(value: i64) -> *mut AMresult {
    AMresult::item(am::Value::int(value).into()).into()
}

/// \memberof AMitem
/// \brief Allocates a new item and initializes it from a null value.
///
/// \return A pointer to an `AMresult` struct with an `AM_VAL_TYPE_NULL` item.
/// \warning The returned `AMresult` struct pointer must be passed to `AMfree()`
///          in order to avoid a memory leak.
#[no_mangle]
pub unsafe extern "C" fn AMitemFromNull() -> *mut AMresult {
    AMresult::item(am::Value::from(()).into()).into()
}

/// \memberof AMitem
/// \brief Allocates a new item and initializes it from a UTF-8 string value.
///
/// \param[in] value A UTF-8 string view as an `AMbyteSpan` struct.
/// \return A pointer to an `AMresult` struct with an `AM_VAL_TYPE_STR` item.
/// \pre \p value.src `!= NULL`
/// \pre `0 <` \p value.count `<= sizeof(`\p value.src `)`
/// \warning The returned `AMresult` struct pointer must be passed to `AMfree()`
///          in order to avoid a memory leak.
/// \internal
///
/// # Safety
/// value.src must be a byte array of length >= value.count
#[no_mangle]
pub unsafe extern "C" fn AMitemFromStr(value: AMbyteSpan) -> *mut AMresult {
    AMresult::item(am::Value::str(to_str!(value)).into()).into()
}

/// \memberof AMitem
/// \brief Allocates a new item and initializes it from a *nix timestamp
///        (milliseconds) value.
///
/// \param[in] value A 64-bit signed integer.
/// \return A pointer to an `AMresult` struct with an `AM_VAL_TYPE_TIMESTAMP` item.
/// \warning The returned `AMresult` struct pointer must be passed to `AMfree()`
///          in order to avoid a memory leak.
#[no_mangle]
pub unsafe extern "C" fn AMitemFromTimestamp(value: i64) -> *mut AMresult {
    AMresult::item(am::Value::timestamp(value).into()).into()
}

/// \memberof AMitem
/// \brief Allocates a new item and initializes it from an unsigned integer value.
///
/// \param[in] value A 64-bit unsigned integer.
/// \return A pointer to an `AMresult` struct with an `AM_VAL_TYPE_UINT` item.
/// \warning The returned `AMresult` struct pointer must be passed to `AMfree()`
///          in order to avoid a memory leak.
#[no_mangle]
pub unsafe extern "C" fn AMitemFromUint(value: u64) -> *mut AMresult {
    AMresult::item(am::Value::uint(value).into()).into()
}

/// \memberof AMitem
/// \brief Gets the type of an item's index.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return An `AMidxType` enum tag.
/// \pre \p item `!= NULL`
/// \post `(`\p item `== NULL) -> 0`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemIdxType(item: *const AMitem) -> AMidxType {
    if let Some(item) = item.as_ref() {
        return item.0.as_ref().into();
    }
    Default::default()
}

/// \memberof AMitem
/// \brief Gets the object identifier of an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return A pointer to an `AMobjId` struct.
/// \pre \p item `!= NULL`
/// \post `(`\p item `== NULL) -> NULL`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemObjId(item: *const AMitem) -> *const AMobjId {
    if let Some(item) = item.as_ref() {
        if let Some(obj_id) = &item.as_ref().obj_id {
            return obj_id;
        }
    }
    std::ptr::null()
}

/// \memberof AMitem
/// \brief Gets the key index of an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return A UTF-8 string view as an `AMbyteSpan` struct.
/// \pre \p item `!= NULL`
/// \post `(`\p item `== NULL) -> (AMbyteSpan){0}`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemKey(item: *const AMitem) -> AMbyteSpan {
    if let Some(item) = item.as_ref() {
        if let Some(index) = &item.as_ref().index {
            if let Ok(key) = index.try_into() {
                return key;
            }
        }
    }
    Default::default()
}

/// \memberof AMitem
/// \brief Gets the position index of an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return A 64-bit unsigned integer.
/// \pre \p item `!= NULL`
/// \post `(`\p item `== NULL) -> SIZE_MAX`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemPos(item: *const AMitem) -> usize {
    if let Some(item) = item.as_ref() {
        if let Some(index) = &item.as_ref().index {
            if let Ok(pos) = index.try_into() {
                return pos;
            }
        }
    }
    usize::MAX
}

/// \memberof AMitem
/// \brief Gets the reference count of an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return A 64-bit unsigned integer.
/// \pre \p item `!= NULL`
/// \post `(`\p item `== NULL) -> 0`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemRefCount(item: *const AMitem) -> usize {
    if let Some(item) = item.as_ref() {
        return Rc::strong_count(&item.0);
    }
    0
}

/// \memberof AMitem
/// \brief Gets a new result for an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return A pointer to an `AMresult` struct.
/// \pre \p item `!= NULL`
/// \post `(`\p item `== NULL) -> NULL`
/// \warning The returned `AMresult` struct pointer must be passed to `AMfree()`
///          in order to avoid a memory leak.
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemResult(item: *const AMitem) -> *mut AMresult {
    if let Some(item) = item.as_ref() {
        return AMresult::item(item.clone()).into();
    }
    std::ptr::null_mut()
}

/// \memberof AMitem
/// \brief Gets the actor identifier value of an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return An `AMactorId` struct pointer.
/// \pre \p item `!= NULL`
/// \pre `AMitemValType(`\p item `) == AM_VAL_TYPE_ACTOR_ID`
/// \post `(`\p item `== NULL) -> NULL`
/// \post `(AMitemValType(`\p item `) != AM_VAL_TYPE_ACTOR_ID) -> NULL`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemToActorId(item: *const AMitem) -> *const AMactorId {
    if let Some(item) = item.as_ref() {
        if let Ok(actor_id) = <&AMactorId>::try_from(item) {
            return actor_id;
        }
    }
    std::ptr::null()
}

/// \memberof AMitem
/// \brief Gets the boolean value of an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return A boolean.
/// \pre \p item `!= NULL`
/// \pre `AMitemValType(`\p item `) == AM_VAL_TYPE_BOOL`
/// \post `(`\p item `== NULL) -> false`
/// \post `(AMitemValType(`\p item `) != AM_VAL_TYPE_BOOL) -> false`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemToBool(item: *const AMitem) -> bool {
    if let Some(item) = item.as_ref() {
        if let Ok(boolean) = item.try_into() {
            return boolean;
        }
    }
    false
}

/// \memberof AMitem
/// \brief Gets the array of bytes value of an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return An `AMbyteSpan` struct.
/// \pre \p item `!= NULL`
/// \pre `AMitemValType(`\p item `) == AM_VAL_TYPE_BYTES`
/// \post `(`\p item `== NULL) -> (AMbyteSpan){0}`
/// \post `(AMitemValType(`\p item `) != AM_VAL_TYPE_BYTES) -> (AMbyteSpan){0}`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemToBytes(item: *const AMitem) -> AMbyteSpan {
    if let Some(item) = item.as_ref() {
        if let Ok(bytes) = item.as_ref().try_into_bytes() {
            return bytes;
        }
    }
    Default::default()
}

/// \memberof AMitem
/// \brief Gets the change value of an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return An `AMchange` struct pointer.
/// \pre \p item `!= NULL`
/// \pre `AMitemValType(`\p item `) == AM_VAL_TYPE_CHANGE`
/// \post `(`\p item `== NULL) -> NULL`
/// \post `(AMitemValType(`\p item `) != AM_VAL_TYPE_CHANGE) -> NULL`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemToChange(item: *mut AMitem) -> *mut AMchange {
    if let Some(item) = item.as_mut() {
        if let Ok(change) = <&mut AMchange>::try_from(item) {
            return change;
        }
    }
    std::ptr::null_mut()
}

/// \memberof AMitem
/// \brief Gets the change hash value of an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return An `AMbyteSpan` struct.
/// \pre \p item `!= NULL`
/// \pre `AMitemValType(`\p item `) == AM_VAL_TYPE_CHANGE_HASH`
/// \post `(`\p item `== NULL) -> (AMbyteSpan){0}`
/// \post `(AMitemValType(`\p item `) != AM_VAL_TYPE_CHANGE_HASH) -> (AMbyteSpan){0}`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemToChangeHash(item: *const AMitem) -> AMbyteSpan {
    if let Some(item) = item.as_ref() {
        if let Ok(change_hash) = item.as_ref().try_into_change_hash() {
            return change_hash;
        }
    }
    Default::default()
}

/// \memberof AMitem
/// \brief Gets the CRDT counter value of an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return A signed 64-bit integer.
/// \pre \p item `!= NULL`
/// \pre `AMitemValType(`\p item `) == AM_VAL_TYPE_COUNTER`
/// \post `(`\p item `== NULL) -> INT64_MAX`
/// \post `(AMitemValType(`\p item `) != AM_VAL_TYPE_COUNTER) -> INT64_MAX`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemToCounter(item: *const AMitem) -> i64 {
    if let Some(item) = item.as_ref() {
        if let Ok(counter) = item.as_ref().try_into_counter() {
            return counter;
        }
    }
    i64::MAX
}

/// \memberof AMitem
/// \brief Gets the document value of an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return An `AMdoc` struct pointer.
/// \pre \p item `!= NULL`
/// \pre `AMitemValType(`\p item `) == AM_VAL_TYPE_DOC`
/// \post `(`\p item `== NULL) -> NULL`
/// \post `(AMitemValType(`\p item `) != AM_VAL_TYPE_DOC) -> NULL`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemToDoc(item: *mut AMitem) -> *mut AMdoc {
    if let Some(item) = item.as_mut() {
        if let Ok(doc) = <&mut AMdoc>::try_from(item) {
            return doc;
        }
    }
    std::ptr::null_mut()
}

/// \memberof AMitem
/// \brief Gets the float value of an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return A 64-bit float.
/// \pre \p item `!= NULL`
/// \pre `AMitemValType(`\p item `) == AM_VAL_TYPE_F64`
/// \post `(`\p item `== NULL) -> DBL_MAX`
/// \post `(AMitemValType(`\p item `) != AM_VAL_TYPE_F64) -> DBL_MAX`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemToF64(item: *const AMitem) -> f64 {
    if let Some(item) = item.as_ref() {
        if let Ok(float) = item.try_into() {
            return float;
        }
    }
    f64::MAX
}

/// \memberof AMitem
/// \brief Gets the integer value of an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return A signed 64-bit integer.
/// \pre \p item `!= NULL`
/// \pre `AMitemValType(`\p item `) == AM_VAL_TYPE_INT`
/// \post `(`\p item `== NULL) -> INT64_MAX`
/// \post `(AMitemValType(`\p item `) != AM_VAL_TYPE_INT) -> INT64_MAX`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemToInt(item: *const AMitem) -> i64 {
    if let Some(item) = item.as_ref() {
        if let Ok(int) = item.as_ref().try_into_int() {
            return int;
        }
    }
    i64::MAX
}

/// \memberof AMitem
/// \brief Gets the UTF-8 string view value of an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return A UTF-8 string view as an `AMbyteSpan` struct.
/// \pre \p item `!= NULL`
/// \pre `AMitemValType(`\p item `) == AM_VAL_TYPE_STR`
/// \post `(`\p item `== NULL) -> (AMbyteSpan){0}`
/// \post `(AMitemValType(`\p item `) != AM_VAL_TYPE_STR) -> (AMbyteSpan){0}`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemToStr(item: *const AMitem) -> AMbyteSpan {
    if let Some(item) = item.as_ref() {
        if let Ok(str) = item.as_ref().try_into_str() {
            return str;
        }
    }
    Default::default()
}

/// \memberof AMitem
/// \brief Gets the synchronization have value of an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return An `AMsyncHave` struct pointer.
/// \pre \p item `!= NULL`
/// \pre `AMitemValType(`\p item `) == AM_VAL_TYPE_SYNC_HAVE`
/// \post `(`\p item `== NULL) -> NULL`
/// \post `(AMitemValType(`\p item `) != AM_VAL_TYPE_SYNC_HAVE) -> NULL`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemToSyncHave(item: *const AMitem) -> *const AMsyncHave {
    if let Some(item) = item.as_ref() {
        if let Ok(sync_have) = <&AMsyncHave>::try_from(item) {
            return sync_have;
        }
    }
    std::ptr::null()
}

/// \memberof AMitem
/// \brief Gets the synchronization message value of an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return An `AMsyncMessage` struct pointer.
/// \pre \p item `!= NULL`
/// \pre `AMitemValType(`\p item `) == AM_VAL_TYPE_SYNC_MESSAGE`
/// \post `(`\p item `== NULL) -> NULL`
/// \post `(AMitemValType(`\p item `) != AM_VAL_TYPE_SYNC_MESSAGE) -> NULL`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemToSyncMessage(item: *const AMitem) -> *const AMsyncMessage {
    if let Some(item) = item.as_ref() {
        if let Ok(sync_message) = <&AMsyncMessage>::try_from(item) {
            return sync_message;
        }
    }
    std::ptr::null()
}

/// \memberof AMitem
/// \brief Gets the synchronization state value of an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return An `AMsyncState` struct pointer.
/// \pre \p item `!= NULL`
/// \pre `AMitemValType(`\p item `) == AM_VAL_TYPE_SYNC_STATE`
/// \post `(`\p item `== NULL) -> NULL`
/// \post `(AMitemValType(`\p item `) != AM_VAL_TYPE_SYNC_STATE) -> NULL`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemToSyncState(item: *mut AMitem) -> *mut AMsyncState {
    if let Some(item) = item.as_mut() {
        if let Ok(sync_state) = <&mut AMsyncState>::try_from(item) {
            return sync_state;
        }
    }
    std::ptr::null_mut()
}

/// \memberof AMitem
/// \brief Gets the *nix timestamp (milliseconds) value of an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return A signed 64-bit integer.
/// \pre \p item `!= NULL`
/// \pre `AMitemValType(`\p item `) == AM_VAL_TYPE_TIMESTAMP`
/// \post `(`\p item `== NULL) -> 0`
/// \post `(AMitemValType(`\p item `) != AM_VAL_TYPE_TIMESTAMP) -> 0`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemToTimestamp(item: *const AMitem) -> i64 {
    if let Some(item) = item.as_ref() {
        if let Ok(timestamp) = item.as_ref().try_into_timestamp() {
            return timestamp;
        }
    }
    0
}

/// \memberof AMitem
/// \brief Gets the unsigned integer value of an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return An unsigned 64-bit integer.
/// \pre \p item `!= NULL`
/// \pre `AMitemValType(`\p item `) == AM_VAL_TYPE_UINT`
/// \post `(`\p item `== NULL) -> UINT64_MAX`
/// \post `(AMitemValType(`\p item `) != AM_VAL_TYPE_UINT) -> UINT64_MAX`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemToUint(item: *const AMitem) -> u64 {
    if let Some(item) = item.as_ref() {
        if let Ok(uint) = item.try_into() {
            return uint;
        }
    }
    u64::MAX
}

/// \memberof AMitem
/// \brief Gets the unknown type of value of an item.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return An `AMunknownValue` struct.
/// \pre \p item `!= NULL`
/// \pre `AMitemValType(`\p item `) == AM_VAL_TYPE_UNKNOWN`
/// \post `(`\p item `== NULL) -> (AMunknownValue){0}`
/// \post `(AMitemValType(`\p item `) != AM_VAL_TYPE_UNKNOWN) -> (AMunknownValue){0}`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemToUnknown(item: *const AMitem) -> AMunknownValue {
    if let Some(item) = item.as_ref() {
        if let Ok(unknown) = item.try_into() {
            return unknown;
        }
    }
    Default::default()
}

/// \memberof AMitem
/// \brief Gets the type of an item's value.
///
/// \param[in] item A pointer to an `AMitem` struct.
/// \return An `AMvalType` enum tag.
/// \pre \p item `!= NULL`
/// \post `(`\p item `== NULL) -> 0`
/// \internal
///
/// # Safety
/// item must be a valid pointer to an AMitem
#[no_mangle]
pub unsafe extern "C" fn AMitemValType(item: *const AMitem) -> AMvalType {
    if let Some(item) = item.as_ref() {
        return item.0.as_ref().into();
    }
    Default::default()
}
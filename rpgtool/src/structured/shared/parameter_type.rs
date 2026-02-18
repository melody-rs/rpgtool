use alox_48::Value;

use crate::structured::{AudioFile, Color, MoveCommand, MoveRoute, Tone};

#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(try_from = "alox_48::Value", try_into = "alox_48::Value")] // TODO make this serde compatible
#[allow(missing_docs)]
pub enum ParameterType {
    Integer(i32),
    String(String),
    Color(Color),
    Tone(Tone),
    AudioFile(AudioFile),
    Float(f64),
    MoveRoute(MoveRoute),
    MoveCommand(MoveCommand),
    Array(Vec<ParameterType>),
    Bool(bool),
    None,
}

impl ParameterType {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(str) => Some(str),
            _ => None,
        }
    }
}

impl TryFrom<alox_48::Value> for ParameterType {
    type Error = alox_48::DeError;

    fn try_from(value: alox_48::Value) -> Result<Self, Self::Error> {
        let result = match value {
            Value::Nil => Self::None,
            Value::Integer(v) => Self::Integer(v),
            Value::Float(v) => Self::Float(v),
            Value::String(v) => {
                let string = String::from_utf8(v.data).map_err(alox_48::DeError::custom)?;
                Self::String(string)
            }
            Value::Array(v) => {
                let array = v
                    .into_iter()
                    .map(<ParameterType as TryFrom<alox_48::Value>>::try_from)
                    .collect::<Result<_, _>>()?;
                Self::Array(array)
            }
            Value::Bool(v) => Self::Bool(v),
            Value::Userdata(userdata) => match userdata.class.as_str() {
                "Color" => Self::Color(Color::from(userdata)),
                "Tone" => Self::Tone(Tone::from(userdata)),
                _ => {
                    return Err(alox_48::DeError::custom(format!(
                        "unknown userdata parameter {userdata:#?}"
                    )));
                }
            },
            Value::Object(alox_48::Object { ref class, .. }) => match class.as_str() {
                "RPG::AudioFile" => Self::AudioFile(alox_48::from_value(&value)?),
                "RPG::MoveRoute" => Self::MoveRoute(alox_48::from_value(&value)?),
                "RPG::MoveCommand" => Self::MoveCommand(alox_48::from_value(&value)?),
                _ => {
                    return Err(alox_48::DeError::custom(format!(
                        "unknown object parameter {value:#?}"
                    )));
                }
            },
            Value::Instance(i) => (*i.value).try_into()?,
            _ => {
                return Err(alox_48::DeError::custom(format!(
                    "unsupported parameter type {value:#?}"
                )));
            }
        };

        Ok(result)
    }
}

impl TryFrom<ParameterType> for alox_48::Value {
    type Error = alox_48::SerError;

    fn try_from(value: ParameterType) -> Result<Self, alox_48::SerError> {
        let value = match value {
            ParameterType::None => Value::Nil,
            ParameterType::Integer(v) => Value::Integer(v),
            ParameterType::Float(v) => Value::Float(v),
            ParameterType::String(v) => Value::String(v.into()),
            ParameterType::Array(v) => {
                let array = v
                    .into_iter()
                    .map(<alox_48::Value as TryFrom<ParameterType>>::try_from)
                    .collect::<Result<_, _>>()?;
                Value::Array(array)
            }
            ParameterType::Bool(v) => Value::Bool(v),
            ParameterType::Color(v) => Value::Userdata(v.into()),
            ParameterType::Tone(v) => Value::Userdata(v.into()),
            ParameterType::AudioFile(v) => alox_48::to_value(v)?,
            ParameterType::MoveRoute(v) => alox_48::to_value(v)?,
            ParameterType::MoveCommand(v) => alox_48::to_value(v)?,
        };
        Ok(value)
    }
}

use alox_48::SerializeArray;
use serde::{de::Error as _, ser::SerializeSeq};

pub struct NilPadded<T>(pub Vec<T>);

impl<T> serde::Serialize for NilPadded<T>
where
    T: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len() + 1))?;
        seq.serialize_element(&())?;

        for v in &self.0 {
            seq.serialize_element(v)?;
        }

        seq.end()
    }
}

impl<T> alox_48::Serialize for NilPadded<T>
where
    T: alox_48::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> alox_48::SerResult<S::Ok>
    where
        S: alox_48::SerializerTrait,
    {
        let mut array = serializer.serialize_array(self.0.len() + 1)?;
        array.serialize_element(&())?;

        for v in &self.0 {
            array.serialize_element(v)?;
        }

        array.end()
    }
}

struct Visitor<T>(std::marker::PhantomData<T>);

impl<'de, T> serde::de::Visitor<'de> for Visitor<T>
where
    T: serde::Deserialize<'de>,
{
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a nil padded array")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let Some(()) = seq.next_element()? else {
            return Err(A::Error::custom("expected first element to be unit"));
        };

        let mut data = vec![];
        while let Some(next) = seq.next_element()? {
            data.push(next);
        }

        Ok(data)
    }
}

impl<'de, T> serde::Deserialize<'de> for NilPadded<T>
where
    T: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_seq(Visitor(std::marker::PhantomData))
            .map(Self)
    }
}

impl<'de, T> alox_48::Visitor<'de> for Visitor<T>
where
    T: alox_48::Deserialize<'de>,
{
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a nil padded array")
    }

    fn visit_array<A>(self, mut seq: A) -> Result<Self::Value, alox_48::DeError>
    where
        A: alox_48::ArrayAccess<'de>,
    {
        let Some(()) = seq.next_element()? else {
            return Err(alox_48::DeError::custom(
                "expected first element to be unit",
            ));
        };

        let mut data = vec![];
        while let Some(next) = seq.next_element()? {
            data.push(next);
        }

        Ok(data)
    }
}

impl<'de, T> alox_48::Deserialize<'de> for NilPadded<T>
where
    T: alox_48::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, alox_48::DeError>
    where
        D: alox_48::DeserializerTrait<'de>,
    {
        deserializer
            .deserialize(Visitor(std::marker::PhantomData))
            .map(Self)
    }
}

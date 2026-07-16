use std::sync::Arc;

use arrow_schema::extension::ExtensionType;
use arrow_schema::{DataType, Field};

use crate::Metadata;

use crate::error::{QuadbinArrowError, QuadbinArrowResult};
use crate::{ QuadbinType};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QuadbinArrowType { 

    QuadbinU64(QuadbinType),
}

impl From<QuadbinArrowType> for DataType {
    fn from(value: QuadbinArrowType) -> Self {
        value.to_data_type()
    }
}

impl QuadbinArrowType {
    /// Returns the [Metadata] contained within this type.
    pub fn metadata(&self) -> &Arc<Metadata> {
        use QuadbinArrowType::*;
        match self {
            QuadbinU64(t) => t.metadata(),
          
           
        }
    }
   
    pub fn to_data_type(&self) -> DataType {
        use QuadbinArrowType::*;
        match self {
            QuadbinU64(_t) => DataType::UInt64,
         
        }
    }


    pub fn to_field<N: Into<String>>(&self, name: N, nullable: bool) -> Field {
        use QuadbinArrowType::*;
        match self {
            QuadbinU64(t) => {
                Field::new(name, self.to_data_type(), nullable).with_extension_type(t.clone())
            }
           
        }
    }



    /// Applies the provided [Metadata] onto self.
    pub fn with_metadata(self, meta: Arc<Metadata>) -> QuadbinArrowType {
        use QuadbinArrowType::*;
        match self {       
            QuadbinU64(t) => QuadbinU64(t.with_metadata(meta)),
          
        }
    }

   
    pub fn from_extension_field(field: &Field) -> QuadbinArrowResult<Self> {
        let extension_name = field.extension_type_name().ok_or(QuadbinArrowError::InvalidGeoArrow(
                "Expected GeoArrow extension metadata, but found none, and `require_geoarrow_metadata` is `true`.".to_string(),
            ))?;

        use QuadbinArrowType::*;
        let data_type = match extension_name {
            QuadbinType::NAME => QuadbinU64(field.try_extension_type()?),
            name => {
                return Err(QuadbinArrowError::InvalidGeoArrow(format!(
                    "Expected a Quadbin extension name, got an Arrow extension type with name: '{name}'.",
                )));
            }
        };
        Ok(data_type)
    }

  
    pub fn from_arrow_field(field: &Field) -> QuadbinArrowResult<Self> {
        use QuadbinArrowType::*;
        if let Ok(geo_type) = Self::from_extension_field(field) {
            Ok(geo_type)
        } else {
            let metadata = Arc::new(Metadata::try_from(field)?);
            let data_type = match field.data_type() {
 
                DataType::UInt64 => QuadbinU64(QuadbinType::new(metadata)),
               
               
                _ => return Err(QuadbinArrowError::InvalidGeoArrow("Only FixedSizeList, Struct, Binary, LargeBinary, BinaryView, String, LargeString, and StringView arrays are unambigously typed for a GeoArrow type and can be used without extension metadata.\nEnsure your array input has GeoArrow metadata.".to_string())),
            };

            Ok(data_type)
        }
    }
}



impl TryFrom<&Field> for QuadbinArrowType {
    type Error = QuadbinArrowError;

    fn try_from(field: &Field) -> QuadbinArrowResult<Self> {
        Self::from_extension_field(field)
    }
}

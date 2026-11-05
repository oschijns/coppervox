use coppervox_common::{
    EMPTY_VOXEL, Index, Model,
    color_space::{ALPHA_THRESHOLD, ColorSpace, LockError},
    rgb::RGB8,
};
use thiserror::Error;
use xcf_rs::data::{
    layer::Layer,
    property::{PropertyIdentifier, PropertyPayload},
    xcf::Xcf,
};

/// Read a GIMP file and convert its layers into voxel slices.
pub fn convert_gimp(
    file: &Xcf,
    color_space: &ColorSpace,
    model: &mut Model,
) -> Result<(), GimpImportError> {
    let cx = file.header.width as Index / 2;
    let cy = file.header.height as Index / 2;
    let cz = file.layers.len() as Index / 2;

    for (layer_index, layer) in file.layers.iter().enumerate() {
        let (offset_x, offset_y) = layer_offsets(layer)?;

        let ox = offset_x as Index - cx;
        let oy = offset_y as Index - cy;

        for x in 0..layer.width {
            for y in 0..layer.height {
                if let Some(pixel) = layer.pixel(x, y) {
                    if pixel.a() < ALPHA_THRESHOLD {
                        continue;
                    }
                    let rgb = RGB8::new(pixel.r(), pixel.g(), pixel.b());
                    let voxel = color_space.identify_color_index(rgb)?;
                    if voxel == EMPTY_VOXEL {
                        continue;
                    }

                    let index = [x as Index - ox, layer_index as Index - cz, y as Index - oy];
                    model[index] = voxel;
                }
            }
        }
    }
    Ok(())
}

fn layer_offsets(layer: &Layer) -> Result<(i32, i32), GimpImportError> {
    let Some(property) = layer
        .properties
        .iter()
        .find(|property| property.kind == PropertyIdentifier::PropOffsets)
    else {
        return Ok((0, 0));
    };

    let (x, y) = match &property.payload {
        PropertyPayload::OffsetsLayer(x, y) => (
            i32::from_ne_bytes(x.to_ne_bytes()),
            i32::from_ne_bytes(y.to_ne_bytes()),
        ),
        // xcf-rs 0.5 currently keeps parsed layer offsets in the raw payload.
        PropertyPayload::Unknown(bytes) if bytes.len() == 8 => (
            i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            i32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        ),
        _ => return Err(GimpImportError::InvalidLayerOffsets(layer.name.clone())),
    };

    Ok((x, y))
}

#[derive(Debug, Error)]
pub enum GimpImportError {
    #[error("Color space: {0}")]
    ColorSpace(#[from] LockError),

    #[error("Invalid layer offsets in GIMP layer \"{0}\"")]
    InvalidLayerOffsets(String),
}

#[cfg(test)]
mod tests {
    use super::{GimpImportError, layer_offsets};
    use xcf_rs::{
        LayerColorType, LayerColorValue,
        data::{
            layer::Layer,
            pixeldata::PixelData,
            property::{Property, PropertyIdentifier, PropertyPayload},
        },
    };

    fn layer_with_offset_payload(payload: PropertyPayload) -> Layer {
        Layer {
            width: 0,
            height: 0,
            kind: LayerColorType {
                kind: LayerColorValue::Rgb,
                alpha: true,
            },
            name: "test".to_owned(),
            properties: vec![Property {
                kind: PropertyIdentifier::PropOffsets,
                length: 8,
                payload,
            }],
            pixels: PixelData {
                width: 0,
                height: 0,
                pixels: vec![],
            },
        }
    }

    #[test]
    fn reads_signed_offsets_from_raw_xcf_property() {
        let layer = layer_with_offset_payload(PropertyPayload::Unknown(vec![
            0xff, 0xff, 0xff, 0xfe, 0x00, 0x00, 0x00, 0x03,
        ]));

        assert_eq!(layer_offsets(&layer).unwrap(), (-2, 3));
    }

    #[test]
    fn reads_signed_offsets_from_typed_property() {
        let layer = layer_with_offset_payload(PropertyPayload::OffsetsLayer(u32::MAX, 4));

        assert_eq!(layer_offsets(&layer).unwrap(), (-1, 4));
    }

    #[test]
    fn rejects_malformed_offset_payload() {
        let layer = layer_with_offset_payload(PropertyPayload::Unknown(vec![0, 1, 2]));

        assert!(matches!(
            layer_offsets(&layer),
            Err(GimpImportError::InvalidLayerOffsets(_))
        ));
    }
}

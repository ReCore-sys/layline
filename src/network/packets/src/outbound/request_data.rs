use bytes::Bytes;
use layline_codec::NetEncode;
use layline_macros::{NetDecode, NetEncode, packet};

#[derive(Debug, Clone, Copy, NetEncode, NetDecode)]
#[starting_index = 1]
pub enum RequestDataType {
    Request,
    FullData,
    PartData,
    PartDataAck,
}

#[derive(Debug, Clone, Copy, NetEncode, NetDecode)]
pub enum RequestDataId {
    ConnectionDetails,
    ZombieOutfitDescriptors,
    PlayerZombieDescriptors,
    RadioData,
    WorldMap,
    PlayerVisited,
}

#[derive(Debug, NetEncode, NetDecode)]
#[packet(name = "RequestData", outbound)]
pub struct RequestData {
    pub request_type: RequestDataType,
    pub data_id: RequestDataId,
    pub payload: Bytes,
}

#[derive(Debug, NetEncode)]
struct PartDataPayload {
    total_size: i32,
    offset: i32,
    part_size: i32,
    data: Bytes,
}

impl RequestData {
    pub fn request(data_id: RequestDataId) -> Self {
        Self {
            request_type: RequestDataType::Request,
            data_id,
            payload: Bytes::new(),
        }
    }

    pub fn full_data(data_id: RequestDataId, payload: Bytes) -> Self {
        Self {
            request_type: RequestDataType::FullData,
            data_id,
            payload,
        }
    }

    pub fn part_data(
        data_id: RequestDataId,
        total_size: usize,
        offset: usize,
        data: &[u8],
    ) -> Self {
        let total_size =
            i32::try_from(total_size).expect("RequestData body exceeds the i32 size limit");
        let offset = i32::try_from(offset).expect("RequestData offset exceeds the i32 size limit");
        let part_size =
            i32::try_from(data.len()).expect("RequestData part exceeds the i32 size limit");

        Self {
            request_type: RequestDataType::PartData,
            data_id,
            payload: PartDataPayload {
                total_size,
                offset,
                part_size,
                data: Bytes::copy_from_slice(data),
            }
            .encode(),
        }
    }

    pub fn part_data_ack(data_id: RequestDataId) -> Self {
        Self {
            request_type: RequestDataType::PartDataAck,
            data_id,
            payload: Bytes::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layline_codec::NetEncode;

    #[test]
    fn encodes_full_data_control_fields_and_body() {
        let packet = RequestData::full_data(
            RequestDataId::ConnectionDetails,
            Bytes::from_static(&[0xaa, 0xbb]),
        );

        assert_eq!(packet.encode().as_ref(), &[2, 0, 0xaa, 0xbb]);
    }

    #[test]
    fn encodes_part_data_header_and_chunk() {
        let packet = RequestData::part_data(
            RequestDataId::ConnectionDetails,
            133_448,
            1_024,
            &[0xaa, 0xbb],
        );

        assert_eq!(
            packet.encode().as_ref(),
            &[3, 0, 0, 2, 9, 72, 0, 0, 4, 0, 0, 0, 0, 2, 0xaa, 0xbb]
        );
    }

    #[test]
    fn encodes_request_and_ack_without_payloads() {
        assert_eq!(
            RequestData::request(RequestDataId::WorldMap)
                .encode()
                .as_ref(),
            &[1, 4]
        );
        assert_eq!(
            RequestData::part_data_ack(RequestDataId::ConnectionDetails)
                .encode()
                .as_ref(),
            &[4, 0]
        );
    }
}

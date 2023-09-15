namespace rs volo.example

struct GetItemRequest {
    1: required i32 opcode,
    2: required string key,
    3: required string value,
}

struct GetItemResponse {
    1: required i32 opcode,
    2: required string key,
    3: required string value,
    4: required bool success,
}

service ItemService {
    GetItemResponse GetItem (1: GetItemRequest req),
    GetItemResponse ServerGetItem (1: GetItemRequest req),
}


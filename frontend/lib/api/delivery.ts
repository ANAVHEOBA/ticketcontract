import { requestText } from "./http";

export const getOpenApiYaml = () => requestText("/docs/openapi.yaml");
export async function getPostmanCollection() {
  try {
    return await requestText("/docs/postman");
  } catch {
    return requestText("/docs/postman_collection.json");
  }
}

export async function getBrunoCollection() {
  try {
    return await requestText("/docs/bruno");
  } catch {
    return requestText("/docs/bruno_collection.json");
  }
}

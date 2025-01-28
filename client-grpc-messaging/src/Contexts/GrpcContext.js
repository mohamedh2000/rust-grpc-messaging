import { ChatClient } from "../proto_js/chat_grpc_web_pb";

//connect to envoy proxy for grpc 
export const client = new ChatClient("http://localhost:8080/", null, null);

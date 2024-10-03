import { createContext } from 'react';
import { io } from "socket.io-client";


// // client-side
// socket.on("connect", () => {
//     console.log(socket.id); // x8WIv7-mJelg7on_ALbx
//   });
export const socket = io("http://localhost:50051/");

socket.on("test", (data) => {
    console.log(data)
})

export const IoContext = createContext();

//socketIo.emit
//socketIO.on
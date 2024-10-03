import { useContext } from 'react';
import { IoContext } from './Contexts/IoContext';
import {client} from './Contexts/GrpcContext'
import { User } from "./proto_js/chat_pb";



const TextBar = () => {
    const io = useContext(IoContext);

    //send message  
    const handleMessage = (e) => {
        console.log(io)
        io.emit("join", "10031")
        io.emit("new message", )
        io.emit("test", "hey!");
        const user = new User();
        user.setUserid('test');
        client.userInfo(user, null, (err, response) => {
          console.log(err, response.toObject());
        }); 

    }

  return (
    <form class="w-10/12 p-4 rounded-3xl absolute mx-auto bottom-0 ">
      <div class="relative">
        <input
          type="search"
          id="default-search"
          class="block rounded-full w-full p-4 ps-10 text-sm text-gray-900 border border-gray-300 bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
          required
        />
        <button
          type="submit"
          class="text-white absolute end-2.5 bottom-2.5 bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-full text-sm px-4 py-2 dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800"
          onClick={handleMessage}
        >
         Send 
        </button>
      </div>
    </form>
  );
};

export default TextBar;

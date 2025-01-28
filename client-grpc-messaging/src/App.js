import './App.css';
import Message from './Message';
import SideBar from './SideBar';
import TextBar from './TextBar';
import { useEffect, useContext } from 'react';
import {client} from './Contexts/GrpcContext'
import { User } from "./proto_js/chat_pb";
import { UserDataContext } from './Contexts/UserDataContext';
function App() {

  //need to get friends and inboxes/unread
  const {setFriends, setRooms, setUsername, 
    setUserPfp, currentRoom, currentMessages} = useContext(UserDataContext);

  useEffect(() => {
    const user = new User();
    user.setUserid('test');
    client.userInfo(user, null, (err, response) => {
      console.log(err, response.toObject());
      let userData = response.toObject();

      if(response) {
        setFriends(userData.userdata.friendsList);
        setRooms(userData.userdata.roomsList);
        setUsername(userData.username);
        setUserPfp(userData.userpfp);
      }
    }); 

  }, []);

  useEffect(() => {

  }, [currentRoom]);


  return (
    <div className="App h-screen w-screen flex">
      <SideBar />
      <ul className="w-full flex h-screen p-4">
        {
          currentMessages.map((message) => <Message userName="test_user_1" message="Tired"/>)
        }
        <TextBar />
      </ul>
    </div>
  );
}

export default App;

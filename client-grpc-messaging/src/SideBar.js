import SideBarButton from "./SideBarButton";
import { useContext} from "react";
import { UserDataContext } from './Contexts/UserDataContext';


const SideBar = () => {
  const {friends, rooms} = useContext(UserDataContext)

  return (
    <div class="relative flex h-full w-full max-w-[15rem] flex-col rounded-xl bg-white bg-clip-border p-4 text-gray-700 shadow-xl shadow-blue-gray-900/5">
      <b>Group Chats</b>
      <nav class="flex min-w-[240px]  flex-col gap-1 p-2 font-sans text-base font-normal text-blue-gray-700">
        {
          rooms.map(
            room => <SideBarButton roomId={room.roomid} text={room.roomname} pfp={room.pfp} />)
        }
      </nav>
      <b>Friends</b>
      <nav class="flex min-w-[240px]  flex-col gap-1 p-2 font-sans text-base font-normal text-blue-gray-700">
        {
          friends.map(friend => <SideBarButton text={"Friend1"}/>)
        } 
      </nav>
    </div>
  );
}

export default SideBar;




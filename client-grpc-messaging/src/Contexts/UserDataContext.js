import { createContext } from 'react';
import { useState } from 'react';

export const UserDataContext = createContext();
export const UserDataProvider = ({ children }) => {
    const [friends, setFriends] = useState([]);
    const [rooms, setRooms] = useState([]);
    const [username, setUsername] = useState('');
    const [userpfp, setUserPfp] = useState('');
    const [currentRoom, setCurrentRoom] = useState();
    const [currentMessages, setMessages] = useState([]);

    return (
        <UserDataContext.Provider value={{ 
            friends, setFriends, rooms, setRooms,
            username, setUsername, userpfp, setUserPfp,
            currentRoom, setCurrentRoom, currentMessages, 
            setMessages
         }} >
            {children}
        </UserDataContext.Provider>
    )

}
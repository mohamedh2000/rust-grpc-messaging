import './App.css';
import Message from './Message';
import SideBar from './SideBar';
import TextBar from './TextBar';

function App() {

  //need to get friends and inboxes/unread  



  return (
    <div className="App h-screen w-screen flex">
      <SideBar />
      <div className="w-full flex h-screen p-4">
        <Message userName="test_user_1" message="Tired"/>
        <TextBar />
      </div>
    </div>
  );
}

export default App;

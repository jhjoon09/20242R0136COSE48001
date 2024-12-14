import { useState, useEffect } from 'react';
import { useNavigate, Routes, Route } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import './App.css';
import Settings from './Setting.tsx';
import Send from './files/Send.tsx';
import Receive from './files/Receive.tsx';
import HomeButton from './component/HomeButton.tsx';
import Dest from './files/Dest.tsx';
import MyDest from './files/MyDest.tsx';
import { appConfigDir, homeDir } from '@tauri-apps/api/path';
import DeviceExplorer from './explorer/DeviceExplorer';

// 메인 페이지
function MainPage() {
  const [greetMsg, setGreetMsg] = useState("");

  const navigate = useNavigate();


  useEffect(() => {
    async function greet() {
      try {
        const nickname = await invoke<String>("get_nickname");
        console.log(nickname);
        setGreetMsg(`Hello, ${nickname}!`);
      } catch (error) {
        console.error("Error fetching nickname:", error);
      }
    }

    async function init() {
      try {
        const is_first_run = await invoke("is_first_run", {savedir : await appConfigDir(), homedir : await homeDir()});
        if (is_first_run) {
          setGreetMsg("Please set your nickname in the settings.");
          navigate("/settings");
          return;
        } 
        await invoke("load_config");
        await greet();
        await invoke("init_client");   
        alert("Server Connected");           
      } catch (error) {
        console.error("Error fetching init:", error);
      }
    }


    init();
  }, [navigate]);

  return (
    <div>
    <HomeButton />
          <button
      onClick={() => navigate("/settings")}
      style={{
        position: "relative", // 고정 위치 설정
        top: "10px",
        left: "10px",
        padding: "10px 15px",
        background: "#f44336",
        color: "white",
        border: "none",
        borderRadius: "5px",
        cursor: "pointer",
      }}
    >
      Setting
    </button>
      <h1>{greetMsg}</h1>
      <button
        onClick={() => navigate("/send")}
        style={{
          margin: "10px",
          padding: "10px 20px",
          cursor: "pointer",
          background: "#4caf50",
          color: "white",
          border: "none",
          borderRadius: "5px",
        }}
      >
        Go to Send
      </button>
      <button
        onClick={() => navigate("/receive")}
        style={{
          margin: "10px",
          padding: "10px 20px",
          cursor: "pointer",
          background: "#2196f3",
          color: "white",
          border: "none",
          borderRadius: "5px",
        }}
      >
        Go to Receive
      </button>
      <button
        onClick={() => navigate('/device-explorer')}
        style={{
          margin: '10px',
          padding: '10px 20px',
          cursor: 'pointer',
          background: '#4caf50',
          color: 'white',
          border: 'none',
          borderRadius: '5px',
        }}
      >
        Go to Device Explorer
      </button>
    </div>
  );
}

// App 컴포넌트
function App() {
  return (
    <Routes>
      <Route path="/" element={<MainPage />} />
      <Route path="/settings" element={<Settings />} />
      <Route path="/send" element={<Send />} />
      <Route path="/dest" element={<Dest />} />
      <Route path="/receive" element={<Receive />} />
      <Route path="/my-dest" element={<MyDest />} />
      <Route path="/device-explorer" element={<DeviceExplorer />} />
    </Routes>
  );
}

export default App;

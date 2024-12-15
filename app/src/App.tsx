import { useState, useEffect } from 'react';
import { useNavigate, Routes, Route } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import './App.css';
import Settings from './Setting.tsx';
import { appConfigDir, homeDir } from '@tauri-apps/api/path';
import DeviceExplorer from './explorer/DeviceExplorer';

// 메인 페이지
function MainPage() {
  const [greetMsg, setGreetMsg] = useState("");
  const [workspace, setWorkspace] = useState("");
  const [isConnected, setIsConnected] = useState(false);
  const navigate = useNavigate();


  useEffect(() => {
    async function greet() {
      try {
        const nickname = await invoke<String>("get_nickname");
        setGreetMsg(`Hello, ${nickname}!`);
        const workspace = await invoke<String>("get_workspace");
        setWorkspace(`Workspace: ${workspace}`);
      } catch (error) {
        console.error("Error fetching nickname:", error);
      }
    }

    async function init() {
      try {
        await invoke("init_client");   
        await greet();
        setIsConnected(true);
      } catch (error) {
        console.error("Error fetching init:", error);
      }
    }

    init();
  }, [navigate]);

  return (
    <div>
      <div>
        <h1>{greetMsg}</h1>
        <h2>{workspace}</h2>
        <h2>{isConnected ? "Connected" : "Connecting to Server"}</h2>
      </div>
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

function Init() {
  const navigate = useNavigate();
  useEffect(() => {
    async function is_first() {
      try {
        const is_first_run = await invoke("is_first_run", {savedir : await appConfigDir(), homedir : await homeDir()});
        if (is_first_run) {
          navigate("/settings", {replace: true});
          return;
        }   
      } catch (error) {
        console.error("Error fetching init:", error);
      }
    }

    is_first();
  }, [navigate]);

  const init_client = async (): Promise<void> => {
    await invoke("load_config");
    navigate("/main", {replace: true});
  }

  return (
    <div>
      <h1>Welcome</h1>
      <div>
        <button
          onClick={() => navigate("/settings", {replace: true})}
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
          Go to Settings
          </button>

        <button
          onClick={() => init_client()}
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
          Go to Home
        </button>

      </div>
    </div>
  );
}

// App 컴포넌트
function App() {
  return (
    <Routes>
      <Route path="/" element={<Init />} />
      <Route path="/main" element={<MainPage />} />
      <Route path="/settings" element={<Settings />} />
      <Route path="/device-explorer" element={<DeviceExplorer />} />
    </Routes>
  );
}

export default App;

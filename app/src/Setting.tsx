import React, { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import HomeButton from "./component/HomeButton";


const Settings: React.FC = () => {
  const [nickname, setNickname] = useState<string>("");
  const [group, setGroup] = useState<string>("");
  const [workspace, setWorkspace] = useState<string>("");
  const navigate = useNavigate();

  const selectDirectory = async () => {
    console.log("Selecting directory...");
    const selected = "~/";

    if (selected) {
      setWorkspace(selected);
    }
  };

  useEffect(() => {

  }, []);

  const saveSetting = async () => {
    try {
      await invoke("set_setting", { workspace, group, nickname }); // Rust에 닉네임과 경로 저장
      alert("Nickname && path saved!");
      navigate("/"); // 메인 페이지로 이동
    } catch (error) {
      console.error("Error saving:", error);
      alert("Failed to save. Try again.");
    }
  };

  return (
    <div style={{ padding: "20px" }}>
      <HomeButton />
      <h1>Settings</h1>
      <div>
        <button
          onClick={() => {
            invoke<string>('print_async', { input: 123 })
              .then((res) => {
                console.log('from rust test_async_from_rust fn :', res);
              })
              .catch((e) => {
                console.error(e);
              });
          }}
          style={{
            position: 'relative', // 고정 위치 설정
            top: '10px',
            left: '10px',
            padding: '10px 15px',
            background: '#f44336',
            color: 'white',
            border: 'none',
            borderRadius: '5px',
            cursor: 'pointer',
          }}
        >
          TestAsyncFromRust
        </button>
      </div>
      <div>
        <label>Workspace Directory:</label>
        <button onClick={selectDirectory}>Select Directory</button>
        <p>Selected Directory: {workspace}</p>
      </div>
      <div>
      <label>
        Enter your nickname:
        <input
          type="text"
          value={nickname}
          onChange={(e) => setNickname(e.target.value)}
          style={{ marginLeft: "10px" }}
        />
      </label>
      </div>
      <div>
      <label>
        Enter your group:
        <input
          type="text"
          value={group}
          onChange={(e) => setGroup(e.target.value)}
          style={{ marginLeft: "10px" }}
        />
      </label>
      </div>
      <br />
      <button onClick={saveSetting} style={{ marginTop: "10px" }}>
        Save
      </button>
    </div>
  );
};

export default Settings;

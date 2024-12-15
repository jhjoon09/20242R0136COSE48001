import React, { useState } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";

interface DirectoryContents {
  folders: string[];
}

const Settings: React.FC = () => {
  const [nickname, setNickname] = useState<string>("");
  const [workspace, setWorkspace] = useState<string>("~");
  const [group, setGroup] = useState<string>("");
  const [openWorkspace, setOpenWorkspace] = useState<boolean>(false);
  const [folders, setFolders] = useState<string[]>([]);
  const [currentPath, setCurrentPath] = useState<string>("~");
  const navigate = useNavigate();

  // 디렉토리 목록 가져오기
  const fetchFolders = async (path: string) => {
    try {
      const data: DirectoryContents = await invoke("get_files", { path });
      setCurrentPath(path);
      setFolders(data.folders);
    } catch (error) {
      console.error("Error fetching folders:", error);
    }
  };

  // 초기 로드
  const handleOpenWorkspace = () => {
    setOpenWorkspace(true);
    fetchFolders("~");
  };

  // 상위 폴더 이동
  const handleGoUp = () => {
    const parentPath = currentPath.split("/").slice(0, -1).join("/") || "~/";
    fetchFolders(parentPath);
  };

  // 폴더 클릭 처리
  const handleFolderClick = (folderName: string) => {
    const newPath = `${currentPath}/${folderName}`;
    fetchFolders(newPath);
  };

  // 폴더 선택 처리
  const handleFolderSelect = (folderName: string) => {
    const selectedPath = `${currentPath}/${folderName}`;
    setWorkspace(selectedPath);
    setOpenWorkspace(false);
  };

  // 설정 저장
  const saveSetting = async () => {
    try {
      await invoke("init_config", { workspace: workspace, group : group, nickname :nickname });
      await invoke("load_config");
      navigate("/main", {replace: true});
    } catch (error) {
      console.error("Error saving:", error);
      alert("Failed to save. Try again.");
    }
  };

  if (openWorkspace) {
    return (
      <div style={{ padding: "20px" }}>
        <h1>Select Workspace Directory</h1>
        <p>Current Path: {currentPath}</p>
        <button onClick={handleGoUp} disabled={currentPath === "~/"}>Go Up</button>
        <ul>
          {folders.map((folder, index) => (
            <li key={index} style={{ display: "flex", alignItems: "center" }}>
              <span
                style={{
                  cursor: "pointer",
                  color: "blue",
                  flex: 1,
                }}
                onClick={() => handleFolderClick(folder)} // 폴더 안으로 이동
              >
                {"📁"} {folder}
              </span>
              <button
                onClick={() => handleFolderSelect(folder)} // 폴더 선택 버튼
                style={{
                  marginLeft: "10px",
                  padding: "5px 10px",
                  backgroundColor: "#28a745",
                  color: "white",
                  border: "none",
                  borderRadius: "5px",
                  cursor: "pointer",
                }}
              >
                Select
              </button>
            </li>
          ))}
        </ul>
      </div>
    );
  }

  return (
    <div style={{ padding: "20px" }}>
      <h1>Settings</h1>
      <div>
        <label>Workspace Directory:</label>
        <button onClick={handleOpenWorkspace}>Select Directory</button>
        <p>Selected Directory: {workspace}</p>
      </div>
      <label>
        Enter your group:
        <input
          type="text"
          value={group}
          onChange={(e) => setGroup(e.target.value)}
          style={{ marginLeft: "10px" }}
        />
      </label>
      <label>
        Enter your nickname:
        <input
          type="text"
          value={nickname}
          onChange={(e) => setNickname(e.target.value)}
          style={{ marginLeft: "10px" }}
        />
      </label>
      <button onClick={saveSetting} style={{ marginTop: "10px" }}>
        Save
      </button>
    </div>
  );
};

export default Settings;

import React, { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import "./Settings.css"; // CSS 파일을 import

interface DirectoryContents {
  folders: string[];
}

const Settings: React.FC = () => {
  const [isFirst, setIsFirst] = useState<boolean>(true);
  const [nickname, setNickname] = useState<string>("");
  const [workspace, setWorkspace] = useState<string>("~");
  const [group, setGroup] = useState<string>("");
  const [openWorkspace, setOpenWorkspace] = useState<boolean>(false);
  const [folders, setFolders] = useState<string[]>([]);
  const [currentPath, setCurrentPath] = useState<string>("~");
  const navigate = useNavigate();

  useEffect(() => {
    const checkFirst = async () => {
      setIsFirst(await invoke("is_first_run"));
    };
    checkFirst();
  }, []);

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

  const goMain = async () => {
    navigate("/", { replace: true });
  }

  // 설정 저장
  const saveSetting = async () => {
    try {
      await invoke("init_config", {
        workspace: workspace,
        group: group,
        nickname: nickname,
      });
      navigate("/", { replace: true });
      if(!isFirst){
        alert("설정을 적용하기 위해서는 재시작이 필요합니다!");
      }
    } catch (error) {
      console.error("Error saving:", error);
      alert("Failed to save. Try again.");
    }
  };

  if (openWorkspace) {
    return (
      <div className="settings-container">
        <h1 className="settings-header">Select Workspace Directory</h1>
        <p className="settings-path">Current Path: {currentPath}</p>
        <button
          className="settings-button"
          onClick={handleGoUp}
          disabled={currentPath === "~/"}
        >
          Go Up
        </button>
        <ul className="settings-folder-list">
          {folders.map((folder, index) => (
            <li key={index} className="settings-folder-item">
              <span
                className="settings-folder-name"
                onClick={() => handleFolderClick(folder)}
              >
                📁 {folder}
              </span>
              <button
                className="settings-select-button"
                onClick={() => handleFolderSelect(folder)}
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
    <div className="settings-container">
      <h1 className="settings-header">
        Settings
        {!isFirst && (
        <button className="settings-cancel-button" onClick={goMain}>
          Cancel
        </button>
        )}
      </h1>
      <div className="settings-input-group">
        <label className="settings-label">Workspace Directory:</label>
        <button className="settings-button" onClick={handleOpenWorkspace}>
          Select Directory
        </button>
        <p className="settings-path">Selected Directory: {workspace}</p>
      </div>
      <div className="settings-input-group">
        <label className="settings-label">Enter your group:</label>
        <input
          type="text"
          value={group}
          onChange={(e) => setGroup(e.target.value)}
          className="settings-input"
        />
      </div>
      <div className="settings-input-group">
        <label className="settings-label">Enter your nickname:</label>
        <input
          type="text"
          value={nickname}
          onChange={(e) => setNickname(e.target.value)}
          className="settings-input"
        />
      </div>
      <button className="settings-save-button" onClick={saveSetting}>
        Save
      </button>
    </div>
  );
};

export default Settings;

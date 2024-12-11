import React, { useState, useEffect } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import HomeButton from "../component/HomeButton";

interface DirectoryContents {
  folders: string[];
}

const MyDest: React.FC = () => {
  const [currentPath, setCurrentPath] = useState<string>("~");
  const [folders, setFolders] = useState<string[]>([]);
  const [selectedFolder, setSelectedFolder] = useState<string | null>(null);
  const [saveName, setSaveName] = useState<string>("");

  const location = useLocation();
  const navigate = useNavigate();

  const { file, nickname, uuid } = location.state || {}; // file과 nickname 데이터를 state로 전달받음

  // 초기 데이터 유효성 검사
  useEffect(() => {
    if (!file || !nickname || !uuid) {
      alert("Missing file or nickname. Returning to the previous page.");
      navigate("/receive");
    }
  }, [file, nickname, navigate]);

  // 디렉토리 목록 가져오기
  const fetchFolders = async (path: string) => {
    try {
      const data: DirectoryContents = await invoke("get_files", { path });
      setCurrentPath(path);
      setFolders(data.folders);
      setSelectedFolder(null);
    } catch (error) {
      console.error("Error fetching folders:", error);
    }
  };

  // 초기 로드
  useEffect(() => {
    fetchFolders("~");
  }, []);

  // 상위 폴더 이동
  const handleGoUp = () => {
    const parentPath = currentPath.split("/").slice(0, -1).join("/") || "~/";
    fetchFolders(parentPath);
  };

  // 폴더 클릭 처리: 폴더 안으로 이동
  const handleFolderClick = (folderName: string) => {
    const newPath = `${currentPath}/${folderName}`;
    fetchFolders(newPath);
  };

  // 폴더 선택 처리
  const handleFolderSelect = (folderName: string) => {
    const folderPath = `${currentPath}/${folderName}`;
    setSelectedFolder(folderPath);
    alert(`Selected folder: ${folderPath}`);
  };

  // 파일 전송 호출
  const confirmAndSend = async () => {
    if (!selectedFolder) {
      alert("Please select a folder.");
      return;
    }

    try {
      await invoke("recive_file", {
        id: uuid,
        source: "." + file.substring(4),
        target: selectedFolder + "/" + saveName ,
      });
      alert(`File transfer initiated from ${file} to ${selectedFolder} by ${nickname}`);
      navigate("/receive"); // 완료 후 다시 receive 페이지로 이동
    } catch (error) {
      console.error("Error invoking recive_file:", error);
      alert("An error occurred during file transfer.");
    }
  };

  return (
    <div style={{ padding: "20px" }}>
      <HomeButton />
      <h1>Select Destination Folder</h1>
      <p>
        <strong>File:</strong> {file}
      </p>
      <p>
        <strong>Nickname:</strong> {nickname}
      </p>
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
      <label>
        Save as:
        <input
          type="text"
          value={saveName}
          onChange={(e) => setSaveName(e.target.value)}
          style={{ marginLeft: "10px" }}
        />
      </label>
      {saveName && selectedFolder && (
        <div>
          <p>Selected Folder: {selectedFolder}</p>
          <button
            onClick={confirmAndSend}
            style={{
              position: "fixed",
              bottom: "20px",
              right: "20px",
              padding: "15px 20px",
              backgroundColor: "#007bff",
              color: "white",
              border: "none",
              borderRadius: "50px",
              boxShadow: "0px 4px 6px rgba(0,0,0,0.1)",
              cursor: "pointer",
            }}
          >
            Confirm and Send
          </button>
        </div>
      )}
    </div>
  );
};

export default MyDest;

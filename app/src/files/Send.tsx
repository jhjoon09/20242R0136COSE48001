import React, { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import HomeButton from "../component/HomeButton";

interface DirectoryContents {
  folders: string[];
  files: string[];
}

const Send: React.FC = () => {
  const [currentPath, setCurrentPath] = useState<string>("~");
  const [folders, setFolders] = useState<string[]>([]);
  const [files, setFiles] = useState<string[]>([]);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);

  const navigate = useNavigate();

  // 디렉토리 데이터 가져오기
  const fetchFiles = async (path: string) => {
    try {
      const data : DirectoryContents = await invoke("get_files", { path });
      
      setCurrentPath(path);
      setFiles(data.files);
      setFolders(data.folders);
      setSelectedFile(null);
    } catch (error) {
      console.error("Error fetching files:", error);
    }
  };

  // 초기 로드
  useEffect(() => {
    fetchFiles("~");
  }, []);

  // 상위 폴더 이동
  const handleGoUp = () => {
    const parentPath = currentPath.split("/").slice(0, -1).join("/") || "~/";
    fetchFiles(parentPath);
  };

  // 폴더 클릭 처리
  const handleFolderClick = (folderName: string) => {
    const newPath = `${currentPath}/${folderName}`;
    fetchFiles(newPath);
  };

  // 파일 클릭 처리
  const handleFileClick = (fileName: string) => {
    setSelectedFile(`${currentPath}/${fileName}`);
  };

  // 파일 전송
  const selectFile = async () => {
    if (selectedFile) {
      try {
        //await invoke("send_file", { from : selectedFile });
        //(`File sent: ${selectedFile}`);
        navigate("/dest", {state: selectedFile });
      } catch (error) {
        console.error("Error sending file:", error);
      }
    }
  };

  return (
    <div style={{ padding: "20px" }}>
      <HomeButton />
      <h1>File Explorer</h1>
      <p>Current Path: {currentPath}</p>
      <button onClick={handleGoUp} disabled={currentPath === "~/"}>
        Go Up
      </button>
      <ul>
        {folders.map((folder, index) => (
          <li
            key={index}
            style={{
              cursor: "pointer",
              color: "blue",
            }}
            onClick={() => handleFolderClick(folder)}
          >
            {"📁"} {folder}
          </li>
        ))}
      </ul>
      <ul>
        {files.map((file, index) => (
          <li
            key={index}
            style={{
              cursor: "pointer",
              color: selectedFile === `${currentPath}/${file}` ? "red" : "black",
            }}
            onClick={() => handleFileClick(file)}
          >
            {"📄"} {file}
          </li>
        ))}
      </ul>
      {selectedFile && (
        <div>
          <p>Selected File: {selectedFile}</p>
          <button
        onClick={selectFile}
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
        SelectFile
          </button>
        </div>
      )}
    </div>
  );
};

export default Send;

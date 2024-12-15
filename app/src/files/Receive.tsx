import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import HomeButton from "../component/HomeButton";

interface FileNode {
  name: string;
  children?: FileNode[];
}

const buildFileTree = (fileList: string[]): FileNode[] => {
  const tree: FileNode[] = [];

  fileList.forEach((filePath) => {
    const parts = filePath.split("/").filter(Boolean); // "/"로 분리하여 경로를 배열로 만듦
    let currentNode = tree;

    parts.forEach((part, index) => {
      let existingNode = currentNode.find((node) => node.name === part);

      if (!existingNode) {
        existingNode = { name: part, children: [] };
        currentNode.push(existingNode);
      }

      // 마지막 부분은 더 이상 자식이 없는 파일이므로 children을 undefined로 설정
      if (index === parts.length - 1) {
        existingNode.children = undefined;
      }

      currentNode = existingNode.children || [];
    });
  });

  return tree;
};

const FileNodeComponent: React.FC<{
  node: FileNode;
  openState: Record<string, boolean>;
  toggleOpen: (path: string) => void;
  path: string;
  onFileSelect: (path: string) => void;
}> = ({ node, openState, toggleOpen, path, onFileSelect }) => {
  const isOpen = openState[path] || false;

  return (
    <li>
      <span onClick={() => toggleOpen(path)} style={{ cursor: "pointer" }}>
        {!node.children ? "✅" : isOpen ? "📂" : "📁"} {node.name}
      </span>
      {isOpen && node.children && (
        <ul>
          {node.children.map((child, index) => (
            <FileNodeComponent
              key={index}
              node={child}
              openState={openState}
              toggleOpen={toggleOpen}
              path={`${path}/${child.name}`}
              onFileSelect={onFileSelect}
            />
          ))}
        </ul>
      )}
      {!node.children && (
        <button onClick={() => onFileSelect(path)} style={{ marginLeft: "10px" }}>
          Select
        </button>
      )}
    </li>
  );
};

const Receive: React.FC = () => {
  const [idMap, setIdMap] = useState<Record<string, [string,string]>>({});
  const [fileMap, setFileMap] = useState<Record<string, string[]> | null>(null);
  const [selectedId, setSelectedId] = useState<string>("");
  const [openState, setOpenState] = useState<Record<string, boolean>>({});
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const navigate = useNavigate();

  useEffect(() => {
    const fetchFiles = async () => {
      try {
        const data = await invoke("get_filemap"); // Rust 함수 호출
        const [files, idmap] = data as [Record<string, string[]>, [[string,string],string][]]
        setFileMap(files);

        const idMapping : Record<string,[string,string]> = {};
        idmap.forEach(([[nickname, uuid],os]) => {
          idMapping[uuid] = [nickname,os];
        });

        setIdMap(idMapping);

      } catch (error) {
        console.error("Error fetching files:", error);
      }
    };

    fetchFiles();
  }, []);

  const toggleOpen = (path: string) => {
    setOpenState((prevState) => ({
      ...prevState,
      [path]: !prevState[path],
    }));
  };

  const handleFileSelect = (path: string) => {
    setSelectedFile(path);
    alert(`Selected file: ${path}`);
    navigate("/my-dest", { state: { file: path, nickname: idMap[selectedId], uuid : selectedId } });
  };

  const handleIdSelect = (uuid: string) => {
    setSelectedId(uuid);
    setOpenState({}); // 닉네임 변경 시 열린 상태 초기화
    setSelectedFile(null); // 닉네임 변경 시 선택된 파일 초기화
  };

  if (!fileMap) {
    return <div>Loading...</div>;
  }

  const fileTree = selectedId ? buildFileTree(fileMap[selectedId]) : [];

  return (
    <div style={{ display: "flex" }}>
      {/* Left Sidebar for Nicknames */}
      <div style={{ width: "200px", padding: "10px", borderRight: "1px solid #ddd" }}>
        <h2>Select Nickname</h2>
        <ul>
          {Object.keys(idMap).map((uuid) => (
            <li key={uuid} onClick={() => handleIdSelect(uuid)} style={{ cursor: "pointer" }}>
              {idMap[uuid][0]} ({idMap[uuid][1]})
            </li>
          ))}
        </ul>
      </div>

      {/* Right Section to Display Files */}
      <div style={{ flex: 1, padding: "10px" }}>
        <HomeButton />
        <h1>Select File</h1>
        <ul>
          {fileTree.map((node, index) => (
            <FileNodeComponent
              key={index}
              node={node}
              openState={openState}
              toggleOpen={toggleOpen}
              path={node.name}
              onFileSelect={handleFileSelect}
            />
          ))}
        </ul>
        {selectedFile && <p>Selected file: {selectedFile}</p>}
      </div>
    </div>
  );
};

export default Receive;

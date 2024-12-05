import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useLocation } from "react-router-dom";
import HomeButton from "../component/HomeButton";

interface FolderNode {
  name: string;
  children?: FolderNode[];
}

const buildFolderTree = (folderList: string[]): FolderNode[] => {
  const tree: FolderNode[] = [];

  // 각 폴더 경로를 "/"로 분리하여 트리 구조를 만든다.
  folderList.forEach((folderPath) => {
    const parts = folderPath.split("/").filter(Boolean); // 경로를 "/"로 분리
    let currentNode = tree;

    parts.forEach((part, index) => {
      let existingNode = currentNode.find((node) => node.name === part);

      if (!existingNode) {
        existingNode = { name: part, children: [] };
        currentNode.push(existingNode);
      }

      // 마지막 부분은 더 이상 자식이 없는 폴더이므로 children을 undefined로 설정
      if (index === parts.length - 1) {
        existingNode.children = undefined;
      }

      currentNode = existingNode.children || [];
    });
  });

  return tree;
};

const FolderNodeComponent: React.FC<{
  node: FolderNode;
  openState: Record<string, boolean>;
  toggleOpen: (path: string) => void;
  path: string;
  onFolderSelect: (path: string) => void;
}> = ({ node, openState, toggleOpen, path, onFolderSelect }) => {
  const isOpen = openState[path] || false;

  return (
    <><HomeButton /><li>
          <span
              onClick={() => toggleOpen(path)}
              style={{ cursor: "pointer" }}
          >
              {!node.children ? "✅" : isOpen ? "📂" : "📁"} {node.name}
              {(
                  <button onClick={() => onFolderSelect(path)} style={{ marginLeft: "10px" }}>
                      Select
                  </button>
              )}
          </span>
          {isOpen && node.children && (
              <ul>
                  {node.children.map((child, index) => (
                      <FolderNodeComponent
                          key={index}
                          node={child}
                          openState={openState}
                          toggleOpen={toggleOpen}
                          path={`${path}/${child.name}`}
                          onFolderSelect={onFolderSelect} />
                  ))}
              </ul>
          )}
      </li></>
  );
};

const Dest: React.FC = () => {
  const [folderMap, setFolderMap] = useState<Record<string, string[]> | null>(null);
  const [selectedFolder, setSelectedFolder] = useState<string | null>(null);
  const [openState, setOpenState] = useState<Record<string, boolean>>({});
  const [file, setFile] = useState<string>("");
  const [nicknames, setNicknames] = useState<string[]>([]); // nickname 리스트 추가
  const [selectedNickname, setSelectedNickname] = useState<string | null>(null); // 선택된 nickname
  const location = useLocation();
  const data = location.state; 

  useEffect(() => {
    const fetchDestinations = async () => {
      try {
        const data = await invoke("get_destinations"); // Call the Tauri command to fetch folder data
        setFolderMap(data as Record<string, string[]>); // Set the fetched folder data

        // Assuming the response has a list of nicknames as well
        setNicknames(Object.keys(data as Record<string, string[]>)); // Set available nicknames from the fetched data
      } catch (error) {
        console.error("Error fetching destinations:", error);
      }
    };

    fetchDestinations();
    setFile(data);
  }, []);

  const toggleOpen = (path: string) => {
    setOpenState((prevState) => ({
      ...prevState,
      [path]: !prevState[path],
    }));
  };

  const handleFolderSelect = (path: string) => {
    setSelectedFolder(path);
    alert(`Selected folder: ${path}`);
    invoke("send_file", {from : file ,id : selectedNickname ,dest : path})
  };

  const handleNicknameSelect = (nickname: string) => {
    setSelectedNickname(nickname);
    alert(`Selected nickname: ${nickname}`);
  };

  if (!folderMap) {
    return <div>Loading...</div>; // Show loading message while data is being fetched
  }

  // Only build tree for the selected nickname
  const folderTree = selectedNickname ? buildFolderTree(folderMap[selectedNickname]) : [];

  return (
    <div style={{ display: "flex" }}>
      {/* Left Sidebar for Nicknames */}
      <div style={{ width: "200px", padding: "10px", borderRight: "1px solid #ddd" }}>
        <h2>Select Nickname</h2>
        <ul>
          {nicknames.map((nickname) => (
            <li key={nickname} onClick={() => handleNicknameSelect(nickname)} style={{ cursor: "pointer" }}>
              {nickname}
            </li>
          ))}
        </ul>
      </div>

      {/* Right Section to Display Folders */}
      <div style={{ flex: 1, padding: "10px" }}>
        <h1>Select Folder</h1>
        <h2>Selected file: {file}</h2>
        <ul>
          {folderTree.map((node, index) => (
            <FolderNodeComponent
              key={index}
              node={node}
              openState={openState}
              toggleOpen={toggleOpen}
              path={node.name}
              onFolderSelect={handleFolderSelect}
            />
          ))}
        </ul>
        {selectedFolder && <p>Selected folder: {selectedFolder}</p>}
      </div>
    </div>
  );
};

export default Dest;

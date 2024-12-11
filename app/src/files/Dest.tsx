import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useLocation, useNavigate } from "react-router-dom";
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

      if(!existingNode.children){
        existingNode.children = [];
      }
      else if (existingNode.children.length === 0 && index === parts.length - 1) {
        existingNode.children = undefined;
      }

      currentNode = existingNode.children!;
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
    <><li>
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
  const [idMap, setIdMap] = useState<Record<string,string>>({});
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [selectedFolder, setSelectedFolder] = useState<string | null>(null);
  const [openState, setOpenState] = useState<Record<string, boolean>>({});
  const [file, setFile] = useState<string>("");
  const [name, setName] = useState<string>("");
  const location = useLocation();
  const data = location.state;
  const navigate = useNavigate(); 

  useEffect(() => {
    const fetchDestinations = async () => {
      try {
        const data = await invoke("get_foldermap"); // Call the Tauri command to fetch folder data
        const [folders, idmap] = data as [Record<string, string[]>, [string,string][]]
        setFolderMap(folders as Record<string, string[]>); // Set the fetched folder data
        const idMapping : Record<string,string> = {};
        idmap.forEach(([nickname, uuid]) => {
          idMapping[uuid] = nickname;
        });

        setIdMap(idMapping);
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
    path = path.substring(4);
    setSelectedFolder(path);
  };

  const handleIdSelect = (uuid: string) => {
    setSelectedId(uuid);

    //alert(`Selected nickname: ${idMap[uuid]}`);
  };

  const sendFile = async () => {
    try{
      if (selectedId) {
        alert(`Send file: my ${file} to ${idMap[selectedId]}'s ${selectedFolder}`);
      }
      const result = await invoke("send_file", {id : selectedId, source : file ,target : (selectedFolder+"/"+name)});
      console.log(result);
      navigate("/");
    }
    catch(e){
      console.log(e);
    }
  }

  if (!folderMap) {
    return <div>Loading...</div>; // Show loading message while data is being fetched
  }

  // Only build tree for the selected nickname
  const folderTree = selectedId ? buildFolderTree(folderMap[selectedId]) : [];

  return (
    <><HomeButton /><div style={{ display: "flex" }}>
      {/* Left Sidebar for Nicknames */}
      <div style={{ width: "200px", padding: "10px", borderRight: "1px solid #ddd" }}>
        <h2>Select Nickname</h2>
        <ul>
          {Object.keys(idMap).map((uuid) => (
            <li key={uuid} onClick={() => handleIdSelect(uuid)} style={{ cursor: "pointer" }}>
              {idMap[uuid]}
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
              onFolderSelect={handleFolderSelect} />
          ))}
        </ul>
        {selectedFolder && <p>Selected folder: {selectedFolder}</p>}
      </div>
    </div>
    <div>
    <label>
        Save as:
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          style={{ marginLeft: "10px" }}
        />
      </label>
      <button onClick={sendFile} style={{ marginTop: "10px" }}>
        send
      </button>
    </div>
    </>
  );
};

export default Dest;

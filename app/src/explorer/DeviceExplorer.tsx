import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState, useCallback, useRef } from 'react';
import FileExplorer from './LocalExplorer';
import RemoteExplorer from './RemoteExplorer';
import DeviceList from './DeviceList';
import './DeviceExplorer.css';
import { Device } from './Types';

const DeviceExplorer: React.FC = () => {
  const [selectedDevice, setSelectedDevice] = useState<Device | null>(null);
  const [devices, setDevices] = useState<Device[]>([]);
  const [folderMap, setFolderMap] = useState<Record<string, string[]>>({});
  const [myDevice, setMyDevice] = useState<Device | null>(null);
  const devicesRef = useRef<Device[]>([]);
  const [isSidebarExpanded, setIsSidebarExpanded] = useState(true);

  useEffect(() => {
    devicesRef.current = devices;
  }, [devices]);

  const updateDevices = useCallback(async () => {
    console.log('updateDevices called');
    try {
      const data = await invoke('get_filemap');
      const [folders, idmap] = data as [
        Record<string, string[]>,
        [[string, string], string][],
      ];

      // 현재의 기기 ID 목록
      const currentDeviceIds = new Set(
        idmap.map(([[nickname, _], _os]) => nickname),
      );

      setDevices((prevDevices) => {
        const updatedDevices: Device[] = [];

        // 기존 기기 상태 업데이트 및 삭제된 기기 오프라인 설정
        prevDevices.forEach((device) => {
          if (currentDeviceIds.has(device.nickname)) {
            // 온라인 상태 업데이트
            updatedDevices.push({
              ...device,
              isOnline: true,
              lastSeen: new Date(),
              isMyDevice: device.nickname === myDevice?.nickname,
            });
          } else {
            // 오프라인 상태로 업데이트
            updatedDevices.push({
              ...device,
              isOnline: false,
              isMyDevice: device.nickname === myDevice?.nickname,
            });
          }
        });

        // 새로운 기기 추가
        idmap.forEach(([[nickname, uuid], os]) => {
          const isExistingDevice = prevDevices.some(
            (device) => device.nickname === nickname,
          );

          if (!isExistingDevice) {
            updatedDevices.push({
              id: uuid,
              nickname,
              os,
              isOnline: true,
              isMyDevice: nickname === myDevice?.nickname,
              lastSeen: new Date(),
            });
          }
        });
        return updatedDevices;
      });

      // 폴더 맵 업데이트
      setFolderMap(folders);
    } catch (error) {
      console.error('Error updating devices:', error);
    }
  }, [myDevice?.nickname]);

  useEffect(() => {
    const initDevices = async () => {
      try {
        const nickname = await invoke<string>('get_nickname');
        setMyDevice({
          id: '',
          nickname,
          os: '',
          isOnline: true,
          isMyDevice: true,
          lastSeen: new Date(),
        });
        await updateDevices();
      } catch (error) {
        console.error('Error initializing devices:', error);
      }
    };

    initDevices();

    const intervalId = setInterval(updateDevices, 10000);
    return () => clearInterval(intervalId);
  }, [updateDevices]);

  return (
    <div className="device-explorer-container flex h-screen bg-gray-100 dark:bg-gray-900">
      {/* 사이드바 */}
      <aside
        className={`transition-all z-10 duration-300 ${
          isSidebarExpanded ? 'w-64' : 'w-20'
        } flex-shrink-0 bg-white dark:bg-gray-800 border-r 
        border-gray-200 dark:border-gray-700`}
      >
        <div className="sidebar-header p-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
          <h2
            className={`text-lg font-semibold text-gray-700 dark:text-gray-200 ${
              isSidebarExpanded ? '' : 'hidden'
            }`}
          >
            기기 목록
          </h2>
          <button
            onClick={() => setIsSidebarExpanded(!isSidebarExpanded)}
            className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-200"
          >
            {isSidebarExpanded ? (
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-6 w-6"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M11 19l-7-7 7-7m8 14l-7-7 7-7"
                />
              </svg>
            ) : (
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-6 w-6"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M13 5l7 7-7 7M5 5l7 7-7 7"
                />
              </svg>
            )}
          </button>
        </div>

        {/* 내 기기 */}
        <div className="device-list-scrollable">
          {myDevice && (
            <div className="p-2 border-b border-gray-200 dark:border-gray-700">
              <button
                onClick={() => setSelectedDevice(myDevice)}
                className={`w-full p-3 rounded-lg text-left transition-colors
                ${
                  selectedDevice?.id === myDevice.id
                    ? 'bg-blue-50 dark:bg-blue-900'
                    : 'hover:bg-gray-50 dark:hover:bg-gray-700'
                }`}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <span className="text-blue-500">📱</span>
                    {isSidebarExpanded && (
                      <span className="font-medium text-gray-700 dark:text-gray-200">
                        현재 기기 ({myDevice.nickname})
                      </span>
                    )}
                  </div>
                  <span className="w-2 h-2 rounded-full bg-green-500" />
                </div>
              </button>
            </div>
          )}
          {/* </div> */}

          {/* 다른 기기들 */}
          {/* <div className="device-list-scrollable"> */}
          <DeviceList
            devices={devices}
            selectedDevice={selectedDevice}
            onDeviceSelect={setSelectedDevice}
            isSidebarExpanded={isSidebarExpanded}
          />
        </div>

        <div className="p-2 mt-auto border-t border-gray-200 dark:border-gray-700">
          <button
            onClick={() => window.history.back()}
            className="w-full p-3 rounded-lg text-left transition-colors hover:bg-gray-50 dark:hover:bg-gray-700"
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2 dark:text-gray-200 ">
                <span className="text-gray-400">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    className="h-6 w-6"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M11 19l-7-7 7-7m8 14l-7-7 7-7"
                    />
                  </svg>
                </span>
                {isSidebarExpanded && (
                  <span className="text-gray-700 dark:text-gray-200">
                    뒤로 가기
                  </span>
                )}
              </div>
            </div>
          </button>
        </div>
      </aside>

      {/* 메인 컨텐츠*/}
      <main className="device-explorer-main">
        {selectedDevice ? (
          selectedDevice.isMyDevice ? (
            <FileExplorer
              myDeviceId={myDevice?.id || null}
              onFileSelect={(path) => console.log('Selected:', path)}
              devices={devices}
            />
          ) : (
            <RemoteExplorer
              curDeviceName={myDevice?.nickname || ''}
              deviceId={selectedDevice.id}
              folderMap={folderMap[selectedDevice.id] || []}
              deviceName={selectedDevice.nickname}
            />
          )
        ) : (
          <div
            className="h-full flex items-center justify-center 
            text-gray-500 dark:text-gray-400"
          >
            왼쪽에서 기기를 선택해주세요
          </div>
        )}
      </main>
    </div>
  );
};

export default DeviceExplorer;

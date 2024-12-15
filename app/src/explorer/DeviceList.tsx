import { Device } from './Types';
import { getOsIcon } from './Types';

interface DeviceListProps {
  devices: Device[];
  selectedDevice: Device | null;
  onDeviceSelect: (device: Device) => void;
  isSidebarExpanded: boolean;
}

const DeviceList: React.FC<DeviceListProps> = ({
  devices,
  selectedDevice,
  onDeviceSelect,
  isSidebarExpanded,
}) => {
  // 온라인/오프라인 기기 분리 (내 기기 제외)
  const onlineDevices = devices.filter(
    (device) => device.isOnline && !device.isMyDevice,
  );
  const offlineDevices = devices.filter(
    (device) => !device.isOnline && !device.isMyDevice,
  );

  return (
    <div className="z-10 relative p-2">
      {/* 온라인 기기 섹션 */}
      <h3 className="px-2 py-1 text-sm text-gray-500 dark:text-gray-400">
        온라인 기기
      </h3>
      <div className="space-y-1 mb-4">
        {onlineDevices.map((device) => (
          <button
            key={device.id}
            onClick={() => onDeviceSelect(device)}
            className={`w-full p-3 rounded-lg text-left transition-colors
              ${
                selectedDevice?.id === device.id
                  ? 'bg-blue-50 dark:bg-blue-900'
                  : 'hover:bg-gray-50 dark:hover:bg-gray-700'
              }`}
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                {getOsIcon(device.os)}
                {isSidebarExpanded && (
                  <span className="text-gray-700 dark:text-gray-200">
                    {device.nickname}
                  </span>
                )}
              </div>
              <span className="w-2 h-2 rounded-full bg-green-500" />
            </div>
          </button>
        ))}
      </div>

      {/* 오프라인 기기 섹션 */}
      {offlineDevices.length > 0 && (
        <>
          <h3 className="px-2 py-1 text-sm text-gray-500 dark:text-gray-400">
            오프라인 기기
          </h3>
          <div className="space-y-1">
            {offlineDevices.map((device) => (
              <button
                key={device.id}
                onClick={() => onDeviceSelect(device)}
                className={`w-full p-3 rounded-lg text-left transition-colors
                  ${
                    selectedDevice?.id === device.id
                      ? 'bg-blue-50 dark:bg-blue-900'
                      : 'hover:bg-gray-50 dark:hover:bg-gray-700'
                  }`}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    {getOsIcon(device.os)}
                    {isSidebarExpanded && (
                      <span className="text-gray-700 dark:text-gray-200">
                        {device.nickname}
                      </span>
                    )}
                  </div>
                  <span className="w-2 h-2 rounded-full bg-gray-300" />
                </div>
              </button>
            ))}
          </div>
        </>
      )}
    </div>
  );
};

export default DeviceList;

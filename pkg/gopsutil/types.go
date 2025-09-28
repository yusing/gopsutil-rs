package gopsutil

type Sensors []TemperatureStat // @name Sensors

type SystemInfo struct {
	Timestamp  int64                         `json:"timestamp"`
	CPUAverage *float64                      `json:"cpu_average"`
	Memory     Memory                        `json:"memory"`
	Disks      map[string]DiskUsageStat      `json:"disks"`    // disk usage by partition
	DisksIO    map[string]DiskIOCountersStat `json:"disks_io"` // disk IO by device
	Network    NetIOCountersStat             `json:"network"`
	Sensors    Sensors                       `json:"sensors"` // sensor temperature by key
} // @name SystemInfo

type Memory struct {
	Total       uint64  `json:"total"`
	Available   uint64  `json:"available"`
	Used        uint64  `json:"used"`
	UsedPercent float32 `json:"used_percent"`
	Free        uint64  `json:"free"`
} // @name Memory

type DiskUsageStat struct {
	Device      string  `json:"device"`
	Path        string  `json:"path"`
	Fs          string  `json:"fstype"`
	Total       uint64  `json:"total"`
	Free        uint64  `json:"free"`
	Used        uint64  `json:"used"`
	UsedPercent float32 `json:"used_percent"`
} // @name DiskUsageStat

type DiskIOCountersStat struct {
	Name string `json:"name"`

	ReadBytes  uint64 `json:"read_bytes"`
	WriteBytes uint64 `json:"write_bytes"`
	ReadCount  uint64 `json:"read_count"`
	WriteCount uint64 `json:"write_count"`

	Iops       uint64  `json:"iops"`        // godoxy
	ReadSpeed  float32 `json:"read_speed"`  // godoxy
	WriteSpeed float32 `json:"write_speed"` // godoxy
} // @name DiskIOCountersStat

type NetIOCountersStat struct {
	BytesSent uint64 `json:"bytes_sent"` // number of bytes sent
	BytesRecv uint64 `json:"bytes_recv"` // number of bytes received

	UploadSpeed   float32 `json:"upload_speed"`   // godoxy
	DownloadSpeed float32 `json:"download_speed"` // godoxy
} // @name NetIOCountersStat

type TemperatureStat struct {
	SensorKey   string  `json:"name"`
	Temperature float32 `json:"temperature"`
	High        float32 `json:"high"`
	Critical    float32 `json:"critical"`
} // @name TemperatureStat

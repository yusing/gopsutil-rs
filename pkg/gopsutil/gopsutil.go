package gopsutil

import (
	"context"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"time"

	"github.com/ebitengine/purego"
	"github.com/yusing/gointernals"
	"github.com/yusing/gointernals/abi"
)

// Library holds the loaded library handle and function pointers
type Library struct {
	handle uintptr

	cpuPercent           func(out *float32) bool
	memory               func(out *Memory) bool
	diskUsage            func(path *string, out *DiskUsageStat) bool
	diskUsageByPartition func(m *gointernals.Map, mType *gointernals.MapType) bool
	diskIOByPartition    func(m *gointernals.Map, mType *gointernals.MapType) bool
	network              func(out *NetIOCountersStat) bool
	temperatures         func(out *Sensors, elemType *abi.Type) bool
}

func New() (*Library, error) {
	libPath, err := getStaticLibraryPath()
	if err != nil {
		return nil, fmt.Errorf("failed to get library path: %w", err)
	}

	// Load the Rust library
	handle, err := purego.Dlopen(libPath, purego.RTLD_NOW|purego.RTLD_GLOBAL)
	if err != nil {
		return nil, fmt.Errorf("failed to load library %s: %w", libPath, err)
	}

	lib := &Library{handle: handle}

	// Register function pointers
	if err := lib.registerFunctions(); err != nil {
		return nil, fmt.Errorf("failed to register functions: %w", err)
	}

	return lib, nil
}

// getStaticLibraryPath returns the path to the dynamic library based on architecture
func getStaticLibraryPath() (string, error) {
	// Look for the library in the target directory first
	targetDir := "target/lib"

	// use native library path if it exists
	nativeLibPath := filepath.Join(targetDir, "native", "libgopsutil_rs.so")
	if _, err := os.Stat(nativeLibPath); !os.IsNotExist(err) {
		return nativeLibPath, nil
	}

	return "", fmt.Errorf("library not found at %s, run 'make rust' to build", nativeLibPath)
}

// registerFunctions registers all the C functions from the Rust library
func (lib *Library) registerFunctions() error {
	// Register CPU functions
	purego.RegisterLibFunc(&lib.cpuPercent, lib.handle, "gopsutil_cpu_percent")

	// Register memory functions
	purego.RegisterLibFunc(&lib.memory, lib.handle, "gopsutil_memory")

	// Register disk usage functions
	purego.RegisterLibFunc(&lib.diskUsage, lib.handle, "gopsutil_disk_usage")

	purego.RegisterLibFunc(&lib.diskUsageByPartition, lib.handle, "gopsutil_disk_usage_by_partition")

	// Register disk IO functions
	purego.RegisterLibFunc(&lib.diskIOByPartition, lib.handle, "gopsutil_disk_io_counters_by_partition")

	// Register network functions
	purego.RegisterLibFunc(&lib.network, lib.handle, "gopsutil_net_io_counters")

	// Register temperatures functions
	purego.RegisterLibFunc(&lib.temperatures, lib.handle, "gopsutil_temperatures")

	return nil
}

// Close closes the library handle
func (lib *Library) Close() error {
	if lib.handle != 0 {
		// Note: purego doesn't provide a dlclose equivalent yet
		lib.handle = 0
	}
	return nil
}

// GetCPUInfo returns comprehensive CPU information
func (lib *Library) GetCPUPercent(ctx context.Context, interval time.Duration) (float32, error) {
	var percent float32
	if !lib.cpuPercent(&percent) {
		return 0, errors.New("failed to get CPU percent")
	}

	select {
	case <-ctx.Done():
		return 0, ctx.Err()
	case <-time.After(interval):
	}

	if !lib.cpuPercent(&percent) {
		return 0, errors.New("failed to get CPU percent")
	}
	return percent, nil
}

func (lib *Library) GetMemoryInfo() (Memory, error) {
	var mem Memory
	if !lib.memory(&mem) {
		return mem, fmt.Errorf("failed to get memory info")
	}
	return mem, nil
}

func (lib *Library) GetDiskUsage(path string) (DiskUsageStat, error) {
	var usage DiskUsageStat
	if !lib.diskUsage(&path, &usage) {
		return usage, fmt.Errorf("failed to get disk usage for path %s", path)
	}
	return usage, nil
}

func (lib *Library) GetDiskUsageByPartition() (map[string]DiskUsageStat, error) {
	m := make(map[string]DiskUsageStat)
	if !lib.diskUsageByPartition(gointernals.MapUnpack(m)) {
		return nil, errors.New("failed to get disk usage by partition")
	}
	return m, nil
}

func (lib *Library) GetDiskIOByPartition() (map[string]DiskIOCountersStat, error) {
	m := make(map[string]DiskIOCountersStat)
	if !lib.diskIOByPartition(gointernals.MapUnpack(m)) {
		return nil, errors.New("failed to get disk IO")
	}
	return m, nil
}

func (lib *Library) GetNetworkInfo() (NetIOCountersStat, error) {
	var net NetIOCountersStat
	if !lib.network(&net) {
		return net, errors.New("failed to get network info")
	}
	return net, nil
}

var tempStatType = func() *abi.Type {
	return gointernals.EfaceOf(TemperatureStat{}).Type
}()

func (lib *Library) GetTemperatures() (Sensors, error) {
	var s Sensors
	if !lib.temperatures(&s, tempStatType) {
		return nil, errors.New("failed to get temperatures")
	}
	return s, nil
}

package gopsutil

import (
	"context"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"time"

	"github.com/ebitengine/purego"
)

// Library holds the loaded library handle and function pointers
type Library struct {
	handle uintptr

	// Function pointers
	cpuPercent           func(out *float32) bool
	memory               func(out *Memory) bool
	diskUsage            func(path *string, out *DiskUsageStat) bool
	diskUsageByPartition func(yield func(*string, *DiskUsageStat)) bool
	diskIOByPartition    func(yield func(*string, *DiskIOCountersStat)) bool
	network              func(out *NetIOCountersStat) bool
	temperatures         func(yield func(*TemperatureStat)) bool
}

func New() (*Library, error) {
	libPath, err := getStaticLibraryPath()
	if err != nil {
		return nil, fmt.Errorf("failed to get library path: %w", err)
	}

	// Load the library
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

	var libPath string
	switch runtime.GOARCH {
	case "amd64":
		libPath = filepath.Join(targetDir, "amd64", "libgopsutil_rs.so")
	case "arm64":
		libPath = filepath.Join(targetDir, "arm64", "libgopsutil_rs.so")
	default:
		return "", fmt.Errorf("GOARCH=%s is not supported", runtime.GOARCH)
	}

	// Check if file exists
	if _, err := os.Stat(libPath); os.IsNotExist(err) {
		return "", fmt.Errorf("library not found at %s, run 'make rust' to build", libPath)
	}

	return libPath, nil
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
	outFn := func(name *string, usage *DiskUsageStat) {
		m[*name] = *usage
	}
	if !lib.diskUsageByPartition(outFn) {
		return nil, errors.New("failed to get disk usage by partition")
	}
	return m, nil
}

func (lib *Library) GetDiskIOByPartition() (map[string]DiskIOCountersStat, error) {
	m := make(map[string]DiskIOCountersStat)
	outFn := func(name *string, io *DiskIOCountersStat) {
		m[*name] = *io
	}
	if !lib.diskIOByPartition(outFn) {
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

func (lib *Library) GetTemperatures() (Sensors, error) {
	m := make(Sensors, 0)
	outFn := func(sensor *TemperatureStat) {
		m = append(m, *sensor)
	}
	if !lib.temperatures(outFn) {
		return nil, errors.New("failed to get temperatures")
	}
	return m, nil
}

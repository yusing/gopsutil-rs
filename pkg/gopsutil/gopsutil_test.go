package gopsutil

import (
	"testing"
	"time"
)

func TestNew(t *testing.T) {
	// Check if library files exist (built with make)
	_, err := getStaticLibraryPath()
	if err != nil {
		t.Skip("Library not built, run 'make rust' to build before testing")
	}

	lib, err := New()
	if err != nil {
		t.Fatalf("Failed to create library instance: %v", err)
	}
	defer lib.Close()

	if lib.handle == 0 {
		t.Error("Library handle should not be zero")
	}
}

func TestCPUFunctions(t *testing.T) {
	lib, err := New()
	if err != nil {
		t.Skip("Library not available, skipping tests")
	}
	defer lib.Close()

	// Test CPU percentage (should be >= 0, not necessarily > 0 if system is idle)
	_, err = lib.GetCPUPercent(t.Context(), 500*time.Millisecond)
	if err != nil {
		t.Errorf("Failed to get CPU percentage: %v", err)
	}
}

func TestMemoryInfo(t *testing.T) {
	lib, err := New()
	if err != nil {
		t.Skip("Library not available, skipping tests")
	}
	defer lib.Close()

	memInfo, err := lib.GetMemoryInfo()
	if err != nil {
		t.Errorf("Failed to get memory info: %v", err)
	}

	if memInfo.Total == 0 {
		t.Error("Expected non-zero total memory")
	}

	if memInfo.Used > memInfo.Total {
		t.Errorf("Used memory (%d) should not exceed total memory (%d)", memInfo.Used, memInfo.Total)
	}

	if memInfo.Available > memInfo.Total {
		t.Errorf("Available memory (%d) should not exceed total memory (%d)", memInfo.Available, memInfo.Total)
	}
}

func TestDiskUsage(t *testing.T) {
	lib, err := New()
	if err != nil {
		t.Skip("Library not available, skipping tests")
	}
	defer lib.Close()

	usage, err := lib.GetDiskUsage("/")
	if err != nil {
		t.Errorf("Failed to get disk usage: %v", err)
	}

	if usage.Total == 0 {
		t.Error("Expected non-zero total disk usage")
	}

	if usage.Used > usage.Total {
		t.Errorf("Used disk usage (%d) should not exceed total disk usage (%d)", usage.Used, usage.Total)
	}

	if usage.Free > usage.Total {
		t.Errorf("Available disk usage (%d) should not exceed total disk usage (%d)", usage.Free, usage.Total)
	}
}

func TestDiskUsageByPartition(t *testing.T) {
	lib, err := New()
	if err != nil {
		t.Skip("Library not available, skipping tests")
	}
	defer lib.Close()

	usage, err := lib.GetDiskUsageByPartition()
	if err != nil {
		t.Errorf("Failed to get disk usage by partition: %v", err)
	}

	if len(usage) == 0 {
		t.Error("Expected non-zero disk usage by partition")
	}

	for _, usage := range usage {
		if usage.Total == 0 {
			t.Error("Expected non-zero total disk usage")
		}
	}

	for _, usage := range usage {
		if usage.Used > usage.Total {
			t.Errorf("Used disk usage (%d) should not exceed total disk usage (%d)", usage.Used, usage.Total)
		}

		if usage.Free > usage.Total {
			t.Errorf("Available disk usage (%d) should not exceed total disk usage (%d)", usage.Free, usage.Total)
		}
	}
}

func TestDiskIOByPartition(t *testing.T) {
	lib, err := New()
	if err != nil {
		t.Skip("Library not available, skipping tests")
	}
	defer lib.Close()

	io, err := lib.GetDiskIOByPartition()
	if err != nil {
		t.Errorf("Failed to get disk IO by partition: %v", err)
	}

	if len(io) == 0 {
		t.Error("Expected non-zero disk IO by partition")
	}
}

func TestNetworkInfo(t *testing.T) {
	lib, err := New()
	if err != nil {
		t.Skip("Library not available, skipping tests")
	}
	defer lib.Close()

	network, err := lib.GetNetworkInfo()
	if err != nil {
		t.Errorf("Failed to get network info: %v", err)
	}

	if network.BytesSent == 0 {
		t.Error("Expected non-zero bytes sent")
	}
}

func TestTemperatures(t *testing.T) {
	lib, err := New()
	if err != nil {
		t.Skip("Library not available, skipping tests")
	}
	defer lib.Close()

	temperatures, err := lib.GetTemperatures()
	if err != nil {
		t.Errorf("Failed to get temperatures: %v", err)
	}

	if len(temperatures) == 0 {
		t.Error("Expected non-zero temperatures")
	}

	for _, temperature := range temperatures {
		if temperature.SensorKey == "" {
			t.Error("Expected non-empty sensor key")
		}
		if temperature.Temperature == 0 {
			t.Error("Expected non-zero temperature")
		}
		if temperature.High == 0 {
			t.Error("Expected non-zero high temperature")
		}
		if temperature.Critical == 0 {
			t.Error("Expected non-zero critical temperature")
		}
	}
}

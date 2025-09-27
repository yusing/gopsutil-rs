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
	if memInfo == nil {
		t.Fatal("Expected non-nil memory info")
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

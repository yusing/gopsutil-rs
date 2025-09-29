package gopsutil

import (
	"C"
	"unsafe"

	_ "github.com/yusing/gointernals"
)

//export StrMapSet
//go:linkname StrMapSet gointernals.StrMapSet
func StrMapSet(m, mType unsafe.Pointer, key string, value unsafe.Pointer)

//export SliceCloneInto
//go:linkname SliceCloneInto gointernals.SliceCloneInto
func SliceCloneInto(dst, src, elemType unsafe.Pointer)

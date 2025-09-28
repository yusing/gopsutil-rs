package gopsutil

import (
	"C"
	"unsafe"

	_ "github.com/yusing/gointernals"
)

//export StrMapSet
//go:linkname StrMapSet gointernals.StrMapSet
func StrMapSet(m unsafe.Pointer, mType unsafe.Pointer, key unsafe.Pointer, value unsafe.Pointer)

//export SliceCloneInto
//go:linkname SliceCloneInto gointernals.SliceCloneInto
func SliceCloneInto(dst unsafe.Pointer, src unsafe.Pointer, elemType unsafe.Pointer)

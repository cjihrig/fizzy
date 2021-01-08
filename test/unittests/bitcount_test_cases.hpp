// Fizzy: A fast WebAssembly interpreter
// Copyright 2021 The Fizzy Authors.
// SPDX-License-Identifier: Apache-2.0

#pragma once

#include <cstdint>
#include <utility>

namespace fizzy::test
{
constexpr std::pair<uint32_t, int> popcount32_test_cases[]{
    {0, 0},
    {1, 1},
    {0x7f, 7},
    {0x80, 1},
    {0x12345678, 13},
    {0xffffffff, 32},
    {0xffff0000, 16},
    {0x0000ffff, 16},
    {0x00ffff00, 16},
    {0x00ff00ff, 16},
    {0x007f8001, 9},
    {0x0055ffaa, 16},
};

constexpr std::pair<uint64_t, int> popcount64_test_cases[]{
    {0, 0},
    {1, 1},
    {0x7f, 7},
    {0x80, 1},
    {0x1234567890abcdef, 32},
    {0xffffffffffffffff, 64},
    {0xffffffff00000000, 32},
    {0x00000000ffffffff, 32},
    {0x0000ffffffff0000, 32},
    {0x00ff00ff00ff00ff, 32},
    {0x007f8001007f8001, 18},
    {0x0055ffaa0055ffaa, 32},
};

constexpr std::pair<uint32_t, int> countl_zero32_test_cases[]{
    {0, 32},
    {1, 31},
    {0x7f, 25},
    {0x80, 24},
    {0x12345678, 3},
    {0xffffffff, 0},
    {0xffff0000, 0},
    {0x0000ffff, 16},
    {0x00ffff00, 8},
    {0x00ff00ff, 8},
    {0x007f8001, 9},
    {0x0055ffaa, 9},
};

constexpr std::pair<uint64_t, int> countl_zero64_test_cases[]{
    {0, 64},
    {1, 63},
    {0x7f, 57},
    {0x80, 56},
    {0x1234567890abcdef, 3},
    {0xffffffffffffffff, 0},
    {0xffffffff00000000, 0},
    {0x00000000ffffffff, 32},
    {0x0000ffffffff0000, 16},
    {0x00ff00ff00ff00ff, 8},
    {0x007f8001007f8001, 9},
    {0x0055ffaa0055ffaa, 9},
};

constexpr std::pair<uint32_t, int> countr_zero32_test_cases[]{
    {0, 32},
    {1, 0},
    {0x7f, 0},
    {0x80, 7},
    {0x12345678, 3},
    {0xffffffff, 0},
    {0xffff0000, 16},
    {0x0000ffff, 0},
    {0x00ffff00, 8},
    {0x00ff00ff, 0},
    {0x007f8001, 0},
    {0x0055ffaa, 1},
};

constexpr std::pair<uint64_t, int> countr_zero64_test_cases[]{
    {0, 64},
    {1, 0},
    {0x7f, 0},
    {0x80, 7},
    {0x1234567890abcdef, 0},
    {0xffffffffffffffff, 0},
    {0xffffffff00000000, 32},
    {0x00000000ffffffff, 0},
    {0x0000ffffffff0000, 16},
    {0x00ff00ff00ff00ff, 0},
    {0x007f8001007f8001, 0},
    {0x0055ffaa0055ffaa, 1},
};
}  // namespace fizzy::test

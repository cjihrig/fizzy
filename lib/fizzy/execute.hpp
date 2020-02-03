#pragma once

#include "types.hpp"
#include <cstdint>

namespace fizzy
{
struct instantiate_error : public std::runtime_error
{
    using runtime_error::runtime_error;
};

// The result of an execution.
struct execution_result
{
    // true if execution resulted in a trap
    bool trapped;
    // the resulting stack (e.g. return values)
    // NOTE: this can be either 0 or 1 items
    std::vector<uint64_t> stack;
};

struct Instance;

using ImportedFunction = execution_result (*)(Instance&, std::vector<uint64_t>);

struct ImportedGlobal
{
    uint64_t* value = nullptr;
    bool is_mutable = false;
};

// The module instance.
struct Instance
{
    Module module;
    bytes memory;
    size_t memory_max_pages = 0;
    std::vector<FuncIdx> table;
    std::vector<uint64_t> globals;
    std::vector<ImportedFunction> imported_functions;
    std::vector<TypeIdx> imported_function_types;
    std::vector<ImportedGlobal> imported_globals;
};

// Instantiate a module.
Instance instantiate(Module module, std::vector<ImportedFunction> imported_functions = {},
    std::vector<ImportedGlobal> imported_globals = {});

// Execute a function on an instance.
execution_result execute(Instance& instance, FuncIdx func_idx, std::vector<uint64_t> args);

// TODO: remove this helper
execution_result execute(const Module& module, FuncIdx func_idx, std::vector<uint64_t> args);

// Find exported function index by name.
std::optional<FuncIdx> find_exported_function(const Module& module, std::string_view name);
}  // namespace fizzy

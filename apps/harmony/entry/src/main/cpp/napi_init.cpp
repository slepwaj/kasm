#include "napi/native_api.h"

#include <cstdint>
#include <cstring>
#include <string>

extern "C" {
void *kasm_create(const char *dict_path);
char *kasm_key(void *handle, uint32_t ch);
char *kasm_backspace(void *handle);
char *kasm_space(void *handle);
char *kasm_select(void *handle, uint32_t index);
int32_t kasm_set_scheme(void *handle, const char *name);
void kasm_destroy(void *handle);
void kasm_free_string(char *text);
}

namespace {

std::string readString(napi_env env, napi_value value) {
    size_t length = 0;
    napi_get_value_string_utf8(env, value, nullptr, 0, &length);
    std::string text(length, '\0');
    napi_get_value_string_utf8(env, value, text.data(), length + 1, &length);
    return text;
}

napi_value fromOwnedString(napi_env env, char *text) {
    if (text == nullptr) {
        napi_value undefined = nullptr;
        napi_get_undefined(env, &undefined);
        return undefined;
    }
    napi_value result = nullptr;
    napi_create_string_utf8(env, text, NAPI_AUTO_LENGTH, &result);
    kasm_free_string(text);
    return result;
}

void *readHandle(napi_env env, napi_value value) {
    int64_t handle = 0;
    napi_get_value_int64(env, value, &handle);
    return reinterpret_cast<void *>(static_cast<intptr_t>(handle));
}

napi_value wrapHandle(napi_env env, void *handle) {
    napi_value result = nullptr;
    napi_create_int64(env, static_cast<int64_t>(reinterpret_cast<intptr_t>(handle)), &result);
    return result;
}

napi_value Create(napi_env env, napi_callback_info info) {
    size_t argc = 1;
    napi_value args[1] = {nullptr};
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 1) {
        return wrapHandle(env, nullptr);
    }
    std::string path = readString(env, args[0]);
    return wrapHandle(env, kasm_create(path.c_str()));
}

napi_value Key(napi_env env, napi_callback_info info) {
    size_t argc = 2;
    napi_value args[2] = {nullptr};
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    uint32_t ch = 0;
    napi_get_value_uint32(env, args[1], &ch);
    return fromOwnedString(env, kasm_key(readHandle(env, args[0]), ch));
}

napi_value Backspace(napi_env env, napi_callback_info info) {
    size_t argc = 1;
    napi_value args[1] = {nullptr};
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    return fromOwnedString(env, kasm_backspace(readHandle(env, args[0])));
}

napi_value Space(napi_env env, napi_callback_info info) {
    size_t argc = 1;
    napi_value args[1] = {nullptr};
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    return fromOwnedString(env, kasm_space(readHandle(env, args[0])));
}

napi_value Select(napi_env env, napi_callback_info info) {
    size_t argc = 2;
    napi_value args[2] = {nullptr};
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    uint32_t index = 0;
    napi_get_value_uint32(env, args[1], &index);
    return fromOwnedString(env, kasm_select(readHandle(env, args[0]), index));
}

napi_value SetScheme(napi_env env, napi_callback_info info) {
    size_t argc = 2;
    napi_value args[2] = {nullptr};
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    std::string name = readString(env, args[1]);
    napi_value result = nullptr;
    napi_get_boolean(env, kasm_set_scheme(readHandle(env, args[0]), name.c_str()) != 0, &result);
    return result;
}

napi_value Destroy(napi_env env, napi_callback_info info) {
    size_t argc = 1;
    napi_value args[1] = {nullptr};
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    kasm_destroy(readHandle(env, args[0]));
    return nullptr;
}

}  // namespace

EXTERN_C_START
static napi_value Init(napi_env env, napi_value exports) {
    napi_property_descriptor desc[] = {
        {"create", nullptr, Create, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"key", nullptr, Key, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"backspace", nullptr, Backspace, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"space", nullptr, Space, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"select", nullptr, Select, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"setScheme", nullptr, SetScheme, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"destroy", nullptr, Destroy, nullptr, nullptr, nullptr, napi_default, nullptr},
    };
    napi_define_properties(env, exports, sizeof(desc) / sizeof(desc[0]), desc);
    return exports;
}
EXTERN_C_END

static napi_module kasmModule = {
    .nm_version = 1,
    .nm_flags = 0,
    .nm_filename = nullptr,
    .nm_register_func = Init,
    .nm_modname = "kasm",
    .nm_priv = ((void *)0),
    .reserved = {0},
};

extern "C" __attribute__((constructor)) void RegisterKasmModule(void) {
    napi_module_register(&kasmModule);
}

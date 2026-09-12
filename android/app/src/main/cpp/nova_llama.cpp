#include "llama.h"
#include <jni.h>
#include <algorithm>
#include <android/log.h>
#include <atomic>
#include <mutex>
#include <string>
#include <vector>

#define LOG_TAG "NovaLlama"
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

namespace {
std::mutex g_mutex;
std::atomic<bool> g_cancel_requested{false};
llama_model * g_model = nullptr;
std::string g_model_path;

std::string jstring_to_string(JNIEnv * env, jstring value) {
    if (value == nullptr) {
        return {};
    }
    const char * chars = env->GetStringUTFChars(value, nullptr);
    if (chars == nullptr) {
        return {};
    }
    std::string result(chars);
    env->ReleaseStringUTFChars(value, chars);
    return result;
}

jstring string_to_jstring(JNIEnv * env, const std::string & value) {
    return env->NewStringUTF(value.c_str());
}

llama_model * get_or_load_model(const std::string & model_path) {
    if (g_model != nullptr && g_model_path == model_path) {
        return g_model;
    }

    if (g_model != nullptr) {
        llama_model_free(g_model);
        g_model = nullptr;
        g_model_path.clear();
    }

    ggml_backend_load_all();

    llama_model_params model_params = llama_model_default_params();
    model_params.n_gpu_layers = 0;
    model_params.load_mode = LLAMA_LOAD_MODE_MMAP;

    g_model = llama_model_load_from_file(model_path.c_str(), model_params);
    if (g_model == nullptr) {
        LOGE("failed to load model: %s", model_path.c_str());
        return nullptr;
    }

    g_model_path = model_path;
    return g_model;
}

std::string generate(const std::string & model_path, const std::string & user_prompt, int max_tokens, int threads) {
    if (model_path.empty() || user_prompt.empty()) {
        return "ERROR: model path or prompt is empty";
    }

    if (max_tokens < 1 || max_tokens > 128) {
        return "ERROR: max_tokens must be between 1 and 128";
    }

    std::lock_guard<std::mutex> lock(g_mutex);
    g_cancel_requested.store(false, std::memory_order_release);

    llama_model * model = get_or_load_model(model_path);
    if (model == nullptr) {
        return "ERROR: unable to load model";
    }

    if (g_cancel_requested.load(std::memory_order_acquire)) {
        return "CANCELLED: generation cancelled";
    }

    const llama_vocab * vocab = llama_model_get_vocab(model);

    const std::string prompt =
        "<|im_start|>system\n"
        "You are NOVA, a small offline Android assistant. Be concise. "
        "Do not claim to have performed an action unless the runtime reports success. "
        "When asked for an OS action, describe the requested intent briefly.\n"
        "<|im_end|>\n"
        "<|im_start|>user\n" + user_prompt +
        "\n<|im_end|>\n<|im_start|>assistant\n";

    const int n_prompt = -llama_tokenize(vocab, prompt.c_str(), prompt.size(), nullptr, 0, true, true);
    if (n_prompt <= 0) {
        return "ERROR: tokenization failed";
    }

    constexpr int kMaxContext = 1024;
    const int requested_context = n_prompt + max_tokens + 8;
    if (requested_context > kMaxContext) {
        return "ERROR: prompt is too long for the mobile context window";
    }

    std::vector<llama_token> prompt_tokens(static_cast<size_t>(n_prompt));
    if (llama_tokenize(vocab, prompt.c_str(), prompt.size(), prompt_tokens.data(), prompt_tokens.size(), true, true) < 0) {
        return "ERROR: tokenization failed";
    }

    if (g_cancel_requested.load(std::memory_order_acquire)) {
        return "CANCELLED: generation cancelled";
    }

    llama_context_params ctx_params = llama_context_default_params();
    ctx_params.n_ctx = static_cast<uint32_t>(requested_context);
    ctx_params.n_batch = static_cast<uint32_t>(n_prompt);
    ctx_params.n_threads = threads > 0 ? threads : 2;
    ctx_params.n_threads_batch = threads > 0 ? threads : 2;
    ctx_params.no_perf = true;

    llama_context * ctx = llama_init_from_model(model, ctx_params);
    if (ctx == nullptr) {
        return "ERROR: unable to create inference context";
    }

    auto sampler_params = llama_sampler_chain_default_params();
    sampler_params.no_perf = true;
    llama_sampler * sampler = llama_sampler_chain_init(sampler_params);
    if (sampler == nullptr) {
        llama_free(ctx);
        return "ERROR: unable to create sampler";
    }
    llama_sampler_chain_add(sampler, llama_sampler_init_greedy());

    llama_batch batch = llama_batch_get_one(prompt_tokens.data(), prompt_tokens.size());
    if (llama_decode(ctx, batch) != 0) {
        llama_sampler_free(sampler);
        llama_free(ctx);
        return "ERROR: prompt evaluation failed";
    }

    std::string output;
    output.reserve(static_cast<size_t>(max_tokens) * 4);

    for (int i = 0; i < max_tokens; ++i) {
        if (g_cancel_requested.load(std::memory_order_acquire)) {
            llama_sampler_free(sampler);
            llama_free(ctx);
            return "CANCELLED: generation cancelled";
        }

        llama_token next_token = llama_sampler_sample(sampler, ctx, -1);
        if (llama_vocab_is_eog(vocab, next_token)) {
            break;
        }

        char piece[256];
        const int n = llama_token_to_piece(vocab, next_token, piece, sizeof(piece), 0, true);
        if (n < 0) {
            break;
        }
        output.append(piece, static_cast<size_t>(n));

        batch = llama_batch_get_one(&next_token, 1);
        if (llama_decode(ctx, batch) != 0) {
            output.append("\n[decode error]");
            break;
        }
    }

    llama_sampler_free(sampler);
    llama_free(ctx);

    return output.empty() ? "" : output;
}
}

extern "C" JNIEXPORT jstring JNICALL
Java_org_nova_os_NativeModelBridge_nativeGenerate(
        JNIEnv * env,
        jclass,
        jstring model_path,
        jstring prompt,
        jint max_tokens,
        jint threads) {
    const std::string path = jstring_to_string(env, model_path);
    const std::string input = jstring_to_string(env, prompt);
    return string_to_jstring(env, generate(path, input, max_tokens, threads));
}

extern "C" JNIEXPORT void JNICALL
Java_org_nova_os_NativeModelBridge_nativeCancel(
        JNIEnv *,
        jclass) {
    g_cancel_requested.store(true, std::memory_order_release);
}

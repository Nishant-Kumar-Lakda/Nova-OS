#include "llama.h"
#include <jni.h>
#include <string>
#include <vector>
#include <mutex>
#include <android/log.h>

#define LOG_TAG "NovaLlama"
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

namespace {
std::mutex g_mutex;

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

std::string generate(const std::string & model_path, const std::string & user_prompt, int max_tokens, int threads) {
    if (model_path.empty() || user_prompt.empty()) {
        return "ERROR: model path or prompt is empty";
    }

    if (max_tokens < 1 || max_tokens > 128) {
        return "ERROR: max_tokens must be between 1 and 128";
    }

    std::lock_guard<std::mutex> lock(g_mutex);

    ggml_backend_load_all();

    llama_model_params model_params = llama_model_default_params();
    model_params.n_gpu_layers = 0;
    model_params.use_mmap = true;

    llama_model * model = llama_model_load_from_file(model_path.c_str(), model_params);
    if (model == nullptr) {
        LOGE("failed to load model: %s", model_path.c_str());
        return "ERROR: unable to load model";
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
        llama_model_free(model);
        return "ERROR: tokenization failed";
    }

    std::vector<llama_token> prompt_tokens(static_cast<size_t>(n_prompt));
    if (llama_tokenize(vocab, prompt.c_str(), prompt.size(), prompt_tokens.data(), prompt_tokens.size(), true, true) < 0) {
        llama_model_free(model);
        return "ERROR: tokenization failed";
    }

    llama_context_params ctx_params = llama_context_default_params();
    ctx_params.n_ctx = static_cast<uint32_t>(std::min(1024, n_prompt + max_tokens + 8));
    ctx_params.n_batch = static_cast<uint32_t>(std::min(n_prompt, 256));
    ctx_params.n_threads = threads > 0 ? threads : 2;
    ctx_params.n_threads_batch = threads > 0 ? threads : 2;
    ctx_params.no_perf = true;

    llama_context * ctx = llama_init_from_model(model, ctx_params);
    if (ctx == nullptr) {
        llama_model_free(model);
        return "ERROR: unable to create inference context";
    }

    auto sampler_params = llama_sampler_chain_default_params();
    sampler_params.no_perf = true;
    llama_sampler * sampler = llama_sampler_chain_init(sampler_params);
    llama_sampler_chain_add(sampler, llama_sampler_init_greedy());

    llama_batch batch = llama_batch_get_one(prompt_tokens.data(), prompt_tokens.size());
    if (llama_decode(ctx, batch) != 0) {
        llama_sampler_free(sampler);
        llama_free(ctx);
        llama_model_free(model);
        return "ERROR: prompt evaluation failed";
    }

    std::string output;
    output.reserve(static_cast<size_t>(max_tokens) * 4);

    for (int i = 0; i < max_tokens; ++i) {
        const llama_token token = llama_sampler_sample(sampler, ctx, -1);
        if (llama_vocab_is_eog(vocab, token)) {
            break;
        }

        char piece[256];
        const int n = llama_token_to_piece(vocab, token, piece, sizeof(piece), 0, true);
        if (n < 0) {
            break;
        }
        output.append(piece, static_cast<size_t>(n));

        batch = llama_batch_get_one(&token, 1);
        if (llama_decode(ctx, batch) != 0) {
            output.append("\n[decode error]");
            break;
        }
    }

    llama_sampler_free(sampler);
    llama_free(ctx);
    llama_model_free(model);

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

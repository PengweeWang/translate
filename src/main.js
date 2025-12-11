const { listen } = window.__TAURI__.event;
const { invoke } = window.__TAURI__.core;
import renderMathInElement from "./assets/katex/contrib/auto-render.mjs";
import axios from "./assets/axios.min.js";

// 用于中断当前翻译请求（防抖/取消）
let currentAbortController = null;
let isAltPressed = false;

async function translateText(text, config) {
  const llm = config.select.llm; 
  const providerConfig = config[llm];

  // 清理 API base
  const apiBase = providerConfig.api_base.trim();
  const apiKey = providerConfig.api_key;
  const model = providerConfig.default_model;
  const temperature = providerConfig.temperature ?? 1.3;

  // 构建翻译 prompt
  const prompt = `请将以下内容准确、流畅地翻译成简体中文：\n\n${text}`;

  const payload = {
    model: model,
    messages: [{ role: "user", content: prompt }],
    temperature: temperature,
    stream: true, // 关键：启用流式
  };

  // 取消上一个请求
  if (currentAbortController) {
    currentAbortController.abort();
  }
  currentAbortController = new AbortController();

  const Dom = document.getElementById("translate-content");
  Dom.innerHTML = ""; // 清空内容，准备流式追加

  try {
    const response = await axios.post(
      `${apiBase}/chat/completions`,
      payload,
      {
        headers: {
          "Authorization": `Bearer ${apiKey}`,
          "Content-Type": "application/json",
        },
        responseType: "stream", // 注意：axios 在浏览器中不支持真正的 stream，但可用 onDownloadProgress 模拟
        onDownloadProgress: (progressEvent) => {
          const chunk = progressEvent.event.target.response;
          if (typeof chunk !== "string") return;

          // 按行分割（SSE 格式：data: {...}\n\ndata: {...}\n\n）
          const lines = chunk.split("\n").filter(line => line.trim() !== "");
          let fullContent = "";

          for (const line of lines) {
            if (line.startsWith("data: ")) {
              const dataStr = line.slice(6); // 去掉 "data: "
              if (dataStr === "[DONE]") break;

              try {
                const parsed = JSON.parse(dataStr);
                const content = parsed.choices?.[0]?.delta?.content || "";
                if (content) {
                  fullContent += content;
                  // 实时更新 DOM
                  Dom.textContent = fullContent;
                }
              } catch (e) {
                console.warn("Failed to parse SSE data:", dataStr, e);
              }
            }
          }
        },
        signal: currentAbortController.signal,
      }
    );
  } catch (error) {
    if (axios.isCancel(error)) {
      console.log("请求被取消");
    } else {
      console.error("翻译出错:", error);
      Dom.textContent = "翻译失败，请检查网络或 API 配置。";
    }
  }
}

async function main() {
  const Dom = document.getElementById("translate-content");
  let config = await invoke('get_config');

  const unlisten = await listen('get_text', async (event) => {
    const originalText = event.payload;
    if (!originalText.trim()) {
      Dom.textContent = "";
      return;
    }


    // 执行翻译
    await translateText(originalText, config);

    // 翻译完成后，渲染数学公式（只在结束时渲染一次，避免卡顿）
    renderMathInElement(Dom, {
      delimiters: [
        { left: "$$", right: "$$", display: true },
        { left: "$", right: "$", display: false },
        { left: "\\(", right: "\\)", display: false },
        { left: "\\[", right: "\\]", display: true }
      ],
      throwOnError: false
    });
  });

  // 可选：返回 unlisten 以便后续清理
  window.unlistenTranslate = unlisten;
}

window.addEventListener("DOMContentLoaded", () => {
  main();
});

window.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') {
    invoke('hide_panel');
  }
});

window.addEventListener('keydown', (e) => {
  if (e.altKey && !isAltPressed) {
    isAltPressed = true;
  }
});

window.addEventListener('keyup', (e) => {
  if (e.key === 'Alt') {
    isAltPressed = false;
  }
});

// 防止 ALT 菜单弹出（可选）
document.addEventListener('mousedown', (e) => {
  if (e.button === 0 && isAltPressed) { // 左键 + ALT
    e.preventDefault();
    // 调用 Tauri 命令开始拖拽
    invoke('start_drag');
  }
});
const { listen } = window.__TAURI__.event;
const { invoke } = window.__TAURI__.core;
import renderMathInElement from "./assets/katex/contrib/auto-render.mjs";

// 用于中断当前翻译请求（防抖/取消）
let currentAbortController = null;
let isAltPressed = false;

async function translateText(text, config) {
  const llm = config.select.llm;
  const providerConfig = config[llm];

  const apiBase = providerConfig.api_base.trim();
  const apiKey = providerConfig.api_key;
  const model = providerConfig.default_model;
  const temperature = providerConfig.temperature ?? 1.3;

  const prompt = `请将以下内容准确、流畅地翻译成简体中文：\n\n${text}`;

  const payload = {
    model: model,
    messages: [{ role: "user", content: prompt }],
    temperature: temperature,
    stream: true,
  };

  // 取消上一个请求
  if (currentAbortController) {
    currentAbortController.abort();
  }
  currentAbortController = new AbortController();

  const Dom = document.getElementById("translate-content");
  Dom.innerHTML = "";

  try {
    const response = await fetch(`${apiBase}/chat/completions`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${apiKey}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(payload),
      signal: currentAbortController.signal,
    });

    if (!response.ok || !response.body) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const reader = response.body.getReader();
    const decoder = new TextDecoder('utf-8');
    let fullContent = "";
    let buffer = "";

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      buffer += decoder.decode(value, { stream: true });

      // 处理可能的多行数据
      let lines = buffer.split('\n');
      buffer = lines.pop() || ""; // 保留不完整的最后一行

      for (const line of lines) {
        if (line.startsWith('data: ')) {
          const dataStr = line.slice(6).trim();
          if (dataStr === '[DONE]') {
            reader.cancel();
            return;
          }

          try {
            const parsed = JSON.parse(dataStr);
            const content = parsed.choices?.[0]?.delta?.content || '';
            if (content) {
              fullContent += content;
              Dom.textContent = fullContent; // 实时更新文本
            }
          } catch (e) {
            console.warn('Failed to parse SSE data:', dataStr, e);
          }
        }
      }
    }
  } catch (error) {
    if (error.name === 'AbortError') {
      console.log('请求被取消');
    } else {
      console.error('翻译出错:', error);
      Dom.textContent = '翻译失败，请检查网络或 API 配置。';
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

// 防止 ALT 菜单弹出
document.addEventListener('mousedown', (e) => {
  if (e.button === 0 && isAltPressed) { // 左键 + ALT
    e.preventDefault();
    // 调用 Tauri 命令开始拖拽
    invoke('start_drag');
  }
});

document.addEventListener('contextmenu', (e) => {
    e.preventDefault();
});
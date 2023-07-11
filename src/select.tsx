import { useEffect, useState } from "react";
import { HomeOutlined, MoreOutlined } from "@ant-design/icons";
import { Button, ButtonGroup } from "@mui/material";
import {
  EmojiPeople,
  Comment,
  Compare,
  Brush,
  FileCopy,
  More,
} from "@mui/icons-material";

import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import { message } from "antd";
// import { ElMessage } from "element-plus";
import "./select.css";
import { createRoot } from "react-dom/client";
import SelectInput from "@mui/material/Select/SelectInput";

interface Mode {
  label: string;
  prompt: string;
  //icon: React.ReactElement;
  icon: React.ReactNode;
  click: (mode: Mode) => void;
}


 export function Select() {
  const [selectedContent, setselectedContent] = useState<string>("");
  const [messageApi, contextHolder] = message.useMessage();
  // <span className="mui-icon mui-icon-paperplane"></span>
  const [modes, setModes] = useState<Mode[]>([
    {
      label: "提问",
      prompt: "",
      icon: <EmojiPeople className="icon" />,
      click: (mode: Mode) => {
        triggerSelectClick(mode.label, mode.prompt);
      },
    },
    {
      label: "解释",
      prompt: "请帮我解释这段文字：",
      icon: <Comment className="icon" />,
      click: (mode: Mode) => {
        triggerSelectClick(mode.label, mode.prompt);
      },
    },
    {
      label: "翻译",
      prompt: "请帮我翻译这段文字：",
      icon: <Compare className="icon" />,
      click: (mode: Mode) => {
        triggerSelectClick(mode.label, mode.prompt);
      },
    },
    {
      label: "润色",
      prompt: "帮我美化或者优化这段文字：",
      icon: <Brush className="icon" />,
      click: (mode: Mode) => {
        triggerSelectClick(mode.label, mode.prompt);
      },
    },
    {
      label: "复制",
      prompt: "",
      icon: <FileCopy className="icon" />,
      click: (mode: Mode) => {
        copySelectContent();
      },
    },
  ]);

  const getSelectedContent = async (): Promise<string> => {
    try {
      const message2 = await invoke("get_selected_content_from_cache");
      console.log("getSelectedContent:", message2);
      return message2 as string; // 使用类型断言将 'unknown' 转换为 'string'
    } catch (e) {
      console.log("getSelectedContent error:");
      console.log(e);
      return ""; // 返回一个默认值（空字符串）或其他适当的错误处理
    }
  };
  
  useEffect(() => {
    getSelectedContent();
  }, []);

  const triggerSelectClick = async (label: string, prompt: string) => {
    const selectedContent = await getSelectedContent();
    console.log("trigger select click", selectedContent);
    const payload = {
      label: label,
      prompt: prompt,
      selected: selectedContent,
    };
    invoke("trigger_select_click", { payload: payload });
    closeSelectWindow();
  };

  const closeSelectWindow = () => {
    invoke("hide_select_window");
  };

  const openSettingsWindow = () => {
    console.log("open settings window");
    invoke("open_setting_window").then(() => {
      closeSelectWindow();
    });
  };

  const copySelectContent = async () => {
    const selectedContent = await getSelectedContent();
    const selected = selectedContent;
    invoke("copy_select_content", { payload: selected });
    // ElMessage({
    //   message: "已复制",
    //   type: "success",
    //   duration: 500,
    // });
    messageApi.open({
      type: "success",
      content: "已复制",
    });
    setTimeout(() => {
      closeSelectWindow();
    }, 700);
  };

  return (
    <div className="container">
      <div className="select-container">
      {modes.map((item) => {
        return (
          <div
            className="select-item"
            key={item.label}
            onClick={() => item.click(item)}
          >
            {item.icon}
            <span className="select-text">{item.label}</span>
          </div>
        );
      })}
      {/* <div className="select-item select-item-min" onClick={() => openSettingsWindow()}>
        <More className="icon" />
      </div> */}
    </div>
    </div>
  );
}

// export default Select;

const root = createRoot(document.getElementById('select')!)

root.render(<Select />)

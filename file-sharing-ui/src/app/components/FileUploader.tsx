"use client";
import { useState } from "react";

export default function FileUploader() {
  const [progress, setProgress] = useState(0);
  const [status, setStatus] = useState("");
  const uploadApi = (process.env.FILE_SERVER_API || "") + "/api/upload";

  const handleUpload = async (e) => {
    const files = e.target.files;

    if (!files.length) {
      return;
    }

    const formData = new FormData();

    // append multiple files if needed
    for (let i = 0; i < files.length; i++) {
      formData.append("file", files[i]);
    }

    const xhr = new XMLHttpRequest();

    xhr.open("POST", uploadApi, true);

    xhr.upload.onprogress = (event) => {
      if (event.lengthComputable) {
        const percentage = (event.loaded / event.total) * 100;
        setProgress(percentage);
      }
    };

    xhr.upload.onload = () => {
      if (xhr.status === 200) {
        setStatus("Upload successful");
      } else {
        setStatus(`Upload failed:  ${xhr.status}`);
      }
    };

    xhr.onerror = () => {
      setStatus("Upload failed");
    };

    xhr.send(formData);
  };

  return (
    <div>
      <input type="file" multiple onChange={handleUpload} />
      <div style={{ width: "100%", background: "#ccc", marginTop: "10px" }}>
        <div
          style={{
            width: `${progress}%`,
            background: "#4caf50",
            height: "20px",
          }}
        ></div>
      </div>
      {status && <p>{status}</p>}
    </div>
  );
}

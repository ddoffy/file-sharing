"use client";
import React, { useState, DragEvent, ChangeEvent } from "react";

export default function DragDropUploader ({ onFilesSelected }) {
  const [dragOver, setDragOver] = useState(false);

  // when a dragged item enters the drop zone
  const handleDragEnter = (e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setDragOver(true);
  };

  // when a dragged item leaves the drop zone
  const handleDragLeave = (e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setDragOver(false);
  };

  // prevent default behavior while dragging over the drop zone
  const handleDragOver = (e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
  };

  // handle file drop
  const handleDrop = (e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setDragOver(false);
    if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
      onFilesSelected(e.dataTransfer.files);
      // clear the drag data cache (for all formats/types)
      e.dataTransfer.clearData();
    }
  };

  // fallback file input change handler
  const handleFileInputChange = (e: ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length > 0) {
      onFilesSelected(e.target.files);
    }
  };

  return (
    <div className="w-full overflow-x-auto">
      <div
        onDragEnter={handleDragEnter}
        onDragLeave={handleDragLeave}
        onDragOver={handleDragOver}
        onDrop={handleDrop}
        style={{
          border: "2px dashed #4caf50",
          borderRadius: "8px",
          padding: "20px",
          textAlign: "center",
          background: dragOver ? "#f3f3f3" : "transparent",
          transition: "background 0.3s ease",
        }}
      >
        <p>Drag & drop your files here</p>
        <p>or</p>
        <input type="file" multiple onChange={handleFileInputChange} />
      </div>
    </div>
  );
};

import { useEffect, useRef, useState } from "react";

interface VideoProgress {
  position: number;
  duration: number;
  percentage: number;
  completed: boolean;
}

export default function VideoPlayer() {
  const videoRef = useRef<HTMLVideoElement>(null);

  const [progress, setProgress] = useState<VideoProgress>({
    position: 0,
    duration: 0,
    percentage: 0,
    completed: false,
  });

  const videoId = "video-123";

  /**
   * Fetch progress from backend
   */
  const fetchProgress = async () => {
    try {
      const response = await fetch(`/api/videos/${videoId}/progress`);
      const data = await response.json();

      setProgress(data);

      if (videoRef.current) {
        videoRef.current.currentTime = data.position;
      }
    } catch (error) {
      console.error(error);
    }
  };

  /**
   * Save progress
   */
  const saveProgress = async (payload: VideoProgress) => {
    try {
      await fetch(`/api/videos/${videoId}/progress`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(payload),
      });
    } catch (error) {
      console.error(error);
    }
  };

  /**
   * Update local state while watching
   */
  const handleTimeUpdate = () => {
    if (!videoRef.current) return;

    const currentTime = videoRef.current.currentTime;
    const duration = videoRef.current.duration;

    const percentage = duration
      ? Number(((currentTime / duration) * 100).toFixed(2))
      : 0;

    const completed = percentage >= 95;

    setProgress({
      position: currentTime,
      duration,
      percentage,
      completed,
    });
  };

  /**
   * Resume playback
   */
  useEffect(() => {
    fetchProgress();
  }, []);

  /**
   * Save every 10 seconds
   */
  useEffect(() => {
    const interval = setInterval(() => {
      saveProgress(progress);
    }, 10000);

    return () => clearInterval(interval);
  }, [progress]);

  /**
   * Save before leaving page
   */
  useEffect(() => {
    const handleBeforeUnload = () => {
      saveProgress(progress);
    };

    window.addEventListener("beforeunload", handleBeforeUnload);

    return () =>
      window.removeEventListener("beforeunload", handleBeforeUnload);
  }, [progress]);

  /**
   * Save when video ends
   */
  const handleEnded = () => {
    const completedProgress = {
      ...progress,
      position: progress.duration,
      percentage: 100,
      completed: true,
    };

    setProgress(completedProgress);
    saveProgress(completedProgress);
  };

  return (
    <div className="max-w-4xl mx-auto p-6">
      <video
        ref={videoRef}
        controls
        className="w-full rounded-lg"
        onTimeUpdate={handleTimeUpdate}
        onEnded={handleEnded}
      >
        <source src="/videos/demo.mp4" type="video/mp4" />
      </video>

      <div className="mt-5 space-y-2">
        <p>Current Position: {Math.floor(progress.position)} sec</p>

        <p>Progress: {progress.percentage}%</p>

        <p>
          Status:
          {progress.completed ? " Completed" : " In Progress"}
        </p>
      </div>
    </div>
  );
}
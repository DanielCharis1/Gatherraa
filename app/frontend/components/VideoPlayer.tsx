import { useEffect, useRef, useState } from "react";

interface VideoPlayerProps {
  videoUrl: string;
  title?: string;
}

export default function VideoPlayer({
  videoUrl,
  title = "Course Video",
}: VideoPlayerProps) {
  const videoRef = useRef<HTMLVideoElement>(null);

  const [playing, setPlaying] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);

  const [volume, setVolume] = useState(1);
  const [muted, setMuted] = useState(false);

  const [playbackRate, setPlaybackRate] = useState(1);

  const formatTime = (seconds: number) => {
    if (!seconds) return "00:00";

    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);

    return `${String(mins).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
  };

  const togglePlay = () => {
    if (!videoRef.current) return;

    if (playing) {
      videoRef.current.pause();
    } else {
      videoRef.current.play();
    }

    setPlaying(!playing);
  };

  const handleSeek = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (!videoRef.current) return;

    const value = Number(e.target.value);

    videoRef.current.currentTime = value;

    setCurrentTime(value);
  };

  const handleVolume = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (!videoRef.current) return;

    const value = Number(e.target.value);

    videoRef.current.volume = value;

    setVolume(value);
  };

  const toggleMute = () => {
    if (!videoRef.current) return;

    videoRef.current.muted = !muted;

    setMuted(!muted);
  };

  const changeSpeed = (speed: number) => {
    if (!videoRef.current) return;

    videoRef.current.playbackRate = speed;

    setPlaybackRate(speed);
  };

  const toggleFullscreen = async () => {
    if (!videoRef.current) return;

    if (document.fullscreenElement) {
      await document.exitFullscreen();
    } else {
      await videoRef.current.requestFullscreen();
    }
  };

  useEffect(() => {
    const video = videoRef.current;

    if (!video) return;

    const update = () => {
      setCurrentTime(video.currentTime);
      setDuration(video.duration || 0);
    };

    const loaded = () => setLoading(false);

    const failed = () => {
      setLoading(false);
      setError(true);
    };

    video.addEventListener("timeupdate", update);
    video.addEventListener("loadedmetadata", loaded);
    video.addEventListener("error", failed);

    return () => {
      video.removeEventListener("timeupdate", update);
      video.removeEventListener("loadedmetadata", loaded);
      video.removeEventListener("error", failed);
    };
  }, []);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    switch (e.key) {
      case " ":
      case "Enter":
        e.preventDefault();
        togglePlay();
        break;

      case "ArrowRight":
        if (videoRef.current)
          videoRef.current.currentTime += 10;
        break;

      case "ArrowLeft":
        if (videoRef.current)
          videoRef.current.currentTime -= 10;
        break;

      case "ArrowUp":
        if (videoRef.current)
          videoRef.current.volume = Math.min(videoRef.current.volume + 0.1, 1);
        break;

      case "ArrowDown":
        if (videoRef.current)
          videoRef.current.volume = Math.max(videoRef.current.volume - 0.1, 0);
        break;

      case "m":
      case "M":
        toggleMute();
        break;
    }
  };

  if (error) {
    return (
      <div className="rounded-lg border border-red-300 bg-red-50 p-8 text-center">
        <p className="font-medium text-red-600">
          Unable to load this video.
        </p>

        <p className="mt-2 text-sm text-gray-600">
          Please check the video URL or try again later.
        </p>
      </div>
    );
  }

  return (
    <div
      tabIndex={0}
      onKeyDown={handleKeyDown}
      className="mx-auto w-full max-w-5xl rounded-lg bg-black p-4 outline-none"
    >
      <h2 className="mb-3 text-white font-semibold">
        {title}
      </h2>

      <div className="relative aspect-video w-full overflow-hidden rounded-lg">
        {loading && (
          <div className="absolute inset-0 flex items-center justify-center bg-black text-white">
            Loading video...
          </div>
        )}

        <video
          ref={videoRef}
          className="h-full w-full"
          src={videoUrl}
        />
      </div>

      <div className="mt-4 space-y-4">

        <input
          type="range"
          min={0}
          max={duration}
          value={currentTime}
          onChange={handleSeek}
          className="w-full"
        />

        <div className="flex flex-wrap items-center gap-3">

          <button
            onClick={togglePlay}
            className="rounded bg-blue-600 px-4 py-2 text-white"
          >
            {playing ? "Pause" : "Play"}
          </button>

          <button
            onClick={toggleMute}
            className="rounded bg-gray-700 px-4 py-2 text-white"
          >
            {muted ? "Unmute" : "Mute"}
          </button>

          <button
            onClick={toggleFullscreen}
            className="rounded bg-gray-700 px-4 py-2 text-white"
          >
            Fullscreen
          </button>

          <div className="flex items-center gap-2">

            <span className="text-white text-sm">
              Volume
            </span>

            <input
              type="range"
              min={0}
              max={1}
              step={0.1}
              value={volume}
              onChange={handleVolume}
            />

          </div>

          <select
            value={playbackRate}
            onChange={(e) =>
              changeSpeed(Number(e.target.value))
            }
            className="rounded p-2"
          >
            <option value={0.5}>0.5x</option>
            <option value={1}>1x</option>
            <option value={1.25}>1.25x</option>
            <option value={1.5}>1.5x</option>
            <option value={2}>2x</option>
          </select>

          <span className="ml-auto text-sm text-white">
            {formatTime(currentTime)} / {formatTime(duration)}
          </span>

        </div>

      </div>
    </div>
  );
}
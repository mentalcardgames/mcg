Improvement for the music statements in the video md features: 

I would like to support the following: 

````
```music
https://link.to/music.mp3
```
````

If the following gets encountered, then use `yt-dlp -x <link>` to download the
video. The file name should be the hash of the link, and it should be downloaded
to the cache. 

In short, the statement should resolve to:

````
```music
~/.cache/instant/video/music/<hash>.mp3
```
````


Make sure you know how yt-dlp (formerly youtube-dl) works, and how to set the
file name. Pay attention that some links might be different audio formats,
either keep them the original format and change the base name, or transcode if
necessary.

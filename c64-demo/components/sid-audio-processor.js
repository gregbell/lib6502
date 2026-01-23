/**
 * SID Audio Processor - AudioWorklet for SID playback
 *
 * This processor runs in a dedicated audio thread, receiving samples from
 * the main thread and playing them back through the Web Audio API.
 *
 * The buffer approach handles the timing mismatch between the emulator's
 * frame-based sample generation and the audio system's block-based processing.
 */

class SidAudioProcessor extends AudioWorkletProcessor {
    constructor() {
        super();

        // Ring buffer for audio samples (about 100ms at 44.1kHz)
        this.bufferSize = 4410;
        this.buffer = new Float32Array(this.bufferSize);
        this.writePos = 0;
        this.readPos = 0;
        this.bufferedSamples = 0;

        // Volume control (0.0 to 1.0)
        this.volume = 0.5;

        // Mute state
        this.muted = false;

        // Handle messages from main thread
        this.port.onmessage = (event) => {
            if (event.data.type === 'samples') {
                // Add samples to buffer
                const samples = event.data.samples;
                for (let i = 0; i < samples.length; i++) {
                    if (this.bufferedSamples < this.bufferSize) {
                        this.buffer[this.writePos] = samples[i];
                        this.writePos = (this.writePos + 1) % this.bufferSize;
                        this.bufferedSamples++;
                    }
                }
            } else if (event.data.type === 'volume') {
                this.volume = event.data.value;
            } else if (event.data.type === 'mute') {
                this.muted = event.data.value;
            } else if (event.data.type === 'clear') {
                // Clear buffer (used on reset)
                this.writePos = 0;
                this.readPos = 0;
                this.bufferedSamples = 0;
                this.buffer.fill(0);
            }
        };
    }

    process(_inputs, outputs, _parameters) {
        const output = outputs[0];
        if (!output || output.length === 0) {
            return true;
        }

        const channel = output[0];
        if (!channel) {
            return true;
        }

        // Fill output buffer with samples from ring buffer
        for (let i = 0; i < channel.length; i++) {
            if (this.muted || this.bufferedSamples === 0) {
                // No samples available or muted - output silence
                channel[i] = 0;
            } else {
                // Output sample with volume applied
                channel[i] = this.buffer[this.readPos] * this.volume;
                this.readPos = (this.readPos + 1) % this.bufferSize;
                this.bufferedSamples--;
            }
        }

        // If stereo output is expected, copy mono to both channels
        if (output.length > 1 && output[1]) {
            output[1].set(channel);
        }

        // Send buffer status back to main thread for debugging
        // (commented out to reduce overhead - uncomment if needed)
        // this.port.postMessage({
        //     type: 'status',
        //     bufferedSamples: this.bufferedSamples,
        //     bufferSize: this.bufferSize
        // });

        // Return true to keep processor alive
        return true;
    }
}

registerProcessor('sid-audio-processor', SidAudioProcessor);

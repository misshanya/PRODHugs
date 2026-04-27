<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Heart } from 'lucide-vue-next'

const props = defineProps<{
  /** Number of heart particles */
  count?: number
}>()

const emit = defineEmits<{
  done: []
}>()

interface Particle {
  id: number
  angle: string
  distance: string
  rotation: string
  endScale: string
  duration: string
  color: string
  size: string
}

const particles = ref<Particle[]>([])

const COLORS = ['#ffdd2d', '#fff705', '#efc800', '#ffe566', '#ffd000']

function rand(min: number, max: number) {
  return min + Math.random() * (max - min)
}

onMounted(() => {
  const total = props.count ?? 12
  const generated: Particle[] = []

  for (let i = 0; i < total; i++) {
    const angleDeg = (360 / total) * i + rand(-20, 20)
    const angleRad = (angleDeg * Math.PI) / 180

    generated.push({
      id: i,
      angle: `${angleRad}rad`,
      distance: `${rand(40, 80)}px`,
      rotation: `${rand(-60, 60)}deg`,
      endScale: `${rand(0.6, 1.2)}`,
      duration: `${rand(600, 900)}ms`,
      color: COLORS[i % COLORS.length]!,
      size: `${rand(0.75, 1.25)}em`,
    })
  }

  particles.value = generated

  // Clean up after longest possible animation
  setTimeout(() => {
    emit('done')
  }, 950)
})
</script>

<template>
  <Heart
    v-for="p in particles"
    :key="p.id"
    class="heart-particle"
    :style="{
      '--angle': p.angle,
      '--distance': p.distance,
      '--rotation': p.rotation,
      '--end-scale': p.endScale,
      '--duration': p.duration,
      '--particle-color': p.color,
      width: p.size,
      height: p.size,
    }"
    fill="currentColor"
  />
</template>

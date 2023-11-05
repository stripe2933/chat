<script setup>
import ConversationPreview from './ConversationPreview.vue';

defineProps({
    conversations: {
        type: Array,
        required: true
    },
    selfUsername: {
        type: String,
        required: true
    },
    selected: {
        required: true
    }
});
defineEmits(['update:selected']);
</script>

<template>
    <div class="flex flex-col overflow-y-auto">
        <ConversationPreview 
            class="hover:bg-gray-800"
            :class="{'bg-gray-900': conversation === selected}"
            v-for="conversation in conversations" 
            :key="conversation.id" 
            :conversation="conversation" 
            :self="selfUsername"
            @click="$emit('update:selected', conversation)"/>
    </div>
</template>
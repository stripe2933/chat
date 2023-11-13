<script setup>
import { computed } from 'vue';
import ConversationThumbnail from './ConversationThumbnail.vue';

const props = defineProps({
    conversation: {
        type: Object,
        required: true
    },
    self: {
        type: String,
        required: true
    }
});

const lastMessagePreview = computed(() => {
    const last_message = props.conversation.last_message;
    if (last_message){
        // Get nickname of sender.
        const sender_username = last_message.sender_username;
        if (sender_username == props.self){
            return `You: ${props.conversation.last_message?.text || ""}`;
        }
        else{
            const sender_nickname = props.conversation.members.find(p => p.username === sender_username)?.nickname;
            return `${sender_nickname}: ${props.conversation.last_message?.text || ""}`;
        }
    }
    else{
        return "";
    }
});
</script>

<template>
    <div class="flex gap-x-2 p-1">
        <ConversationThumbnail class="min-w-[4rem] w-16 h-16" :members="conversation.members" :selfUsername="self"/>
        <div class="flex flex-col justify-center gap-y-1 overflow-hidden">
            <p class="text-gray-100 text-ellipsis font-bold truncate">{{ conversation.name }}</p>
            <p class="text-gray-400 text-ellipsis text-sm truncate">{{ lastMessagePreview }}</p>
        </div>
    </div>
</template>
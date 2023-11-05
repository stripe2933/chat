<script setup>
import ConversationList from './ConversationList.vue';
import Conversation from './Conversation.vue';
import ProfileBadge from '../ProfileBadge.vue';

import { onMounted, ref } from 'vue';

const self = ref(null);

const joinedConversations = ref([]);
const selectedConversation = ref(null);

const usersExceptSelf = ref([]);
const newConversationDialogVisible = ref(false);
const newConversationName = ref('');
const newConversationMembers = ref([]);

onMounted(async () => {
    const selfResponse = await fetch('https://localhost:8443/api/user/login_info', { mode: 'cors', credentials: 'include' });
    self.value = await selfResponse.json();

    const usersResponse = await fetch('https://localhost:8443/api/user/all');
    usersExceptSelf.value = (await usersResponse.json())
        .filter(user => user.username !== self.value.username);

    const joinedConversationsResponse = await fetch('https://localhost:8443/api/conversation/joined', { mode: 'cors', credentials: 'include' });
    joinedConversations.value = await joinedConversationsResponse.json();
})

function openNewConversationDialog() {
    newConversationDialogVisible.value = true;
}

async function submitNewConversation(event){
    alert(newConversationName.value);
    alert(newConversationMembers.value);

    newConversationDialogVisible.value = false;

}
</script>

<template>
    <div class="flex flex-col justify-stretch bg-gray-950 h-screen max-h-screen">
        <header class="self-stretch flex items-center">
            <div class="flex-1"></div>
            <h1 class="flex-1 text-center text-3xl font-bold text-white m-4">Chat</h1>
            <div class="flex-1 flex justify-end">
                <ProfileBadge v-if="self" class="m-2" :user="self"/>
            </div>
        </header>

        <hr class="bg-gray-600">

        <main v-if="self" class="grow flex justify-stretch items-stretch overflow-y-auto">
            <section class="flex flex-col gap-y-2 max-w-[30%] p-4">
                <div class="flex justify-between gap-x-4">
                    <p class="text-gray-100 text-lg font-bold">Conversations</p>
                    <button class="bg-blue-600 text-gray-200 text-xs px-2 rounded-lg"
                        @click="(_) => openNewConversationDialog()">New</button>
                </div>
                <ConversationList :conversations="joinedConversations" :selfUsername="self.username" v-model:selected="selectedConversation" />
            </section>

            <!-- Vertical separator -->
            <div class="w-[1px] bg-gray-600"></div>

            <section class="grow p-4 flex justify-stretch items-stretch overflow-y-auto">
                <Conversation v-if="selectedConversation" :conversation_id="selectedConversation.id" :self="self" />

                <div v-else class="grow flex flex-col justify-center items-center">
                    <img class="w-48" src="src/assets/speech-bubble.png" alt="Conversations icon">
                    <p class="text-gray-100">Select a conversation to start</p>
                </div>
            </section>
        </main>
    </div>

    <v-dialog v-model="newConversationDialogVisible" width="500">
        <template v-slot:default="{ isActive }">
            <v-card title="Create new conversation">
                <v-form @submit.prevent="submitNewConversation">
                    <v-container>
                        <v-row>
                            <v-col cols="12">
                                <v-text-field v-model="newConversationName" label="Conversation name" required />
                                <v-autocomplete v-model="newConversationMembers" label="Members" :items="usersExceptSelf" item-title="nickname" item-value="username" required multiple chips>
                                    <template v-slot:chip="{ props, item }">
                                        <v-chip v-bind="props" :prepend-avatar="`https://localhost:8443/api/user/profile_picture/${item.raw.profile_picture_filename}`"
                                            :text="item.raw.nickname" />
                                    </template>
                                </v-autocomplete>
                                <v-btn type="submit" text="Create..."/>
                            </v-col>
                        </v-row>
                    </v-container>
                </v-form>
            </v-card>
        </template>
    </v-dialog>
</template>
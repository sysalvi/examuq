<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('exam_sessions', function (Blueprint $table): void {
            $table->index(['exam_id', 'status', 'started_at'], 'exam_sessions_exam_status_started_idx');
            $table->index(['status', 'last_heartbeat_at'], 'exam_sessions_status_heartbeat_idx');
            $table->index(['status', 'ended_at'], 'exam_sessions_status_ended_idx');
        });
    }

    public function down(): void
    {
        Schema::table('exam_sessions', function (Blueprint $table): void {
            $table->dropIndex('exam_sessions_exam_status_started_idx');
            $table->dropIndex('exam_sessions_status_heartbeat_idx');
            $table->dropIndex('exam_sessions_status_ended_idx');
        });
    }
};

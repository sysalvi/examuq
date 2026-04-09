<?php

namespace App\Filament\Widgets;

use App\Models\Exam;
use App\Models\ExamSession;
use Carbon\Carbon;
use Filament\Widgets\StatsOverviewWidget;
use Filament\Widgets\StatsOverviewWidget\Stat;

class SessionOverviewWidget extends StatsOverviewWidget
{
    protected ?string $pollingInterval = '20s';

    protected function getStats(): array
    {
        $now = Carbon::now();
        $todayStart = $now->copy()->startOfDay();

        $activeSessions = ExamSession::query()->where('status', 'active')->count();

        $staleSessions = ExamSession::query()
            ->where('status', 'active')
            ->where(function ($query) use ($now): void {
                $query
                    ->whereNull('last_heartbeat_at')
                    ->orWhere('last_heartbeat_at', '<=', $now->copy()->subMinutes(2));
            })
            ->count();

        $finishedToday = ExamSession::query()
            ->where('status', 'finished')
            ->where('ended_at', '>=', $todayStart)
            ->count();

        $activeExams = Exam::query()->where('is_active', true)->count();

        return [
            Stat::make('Sesi Aktif', (string) $activeSessions)
                ->description('Monitoring live saat ini')
                ->descriptionIcon('heroicon-m-signal')
                ->color('success'),
            Stat::make('Sesi Stale', (string) $staleSessions)
                ->description('Heartbeat > 2 menit')
                ->descriptionIcon('heroicon-m-exclamation-triangle')
                ->color($staleSessions > 0 ? 'warning' : 'gray'),
            Stat::make('Sesi Selesai Hari Ini', (string) $finishedToday)
                ->description('Total selesai per hari')
                ->descriptionIcon('heroicon-m-check-badge')
                ->color('info'),
            Stat::make('Ujian Aktif', (string) $activeExams)
                ->description('Siap dipakai login siswa')
                ->descriptionIcon('heroicon-m-academic-cap')
                ->color('primary'),
        ];
    }
}
